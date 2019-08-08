package io.github.bennofs;

import com.github.luben.zstd.ZstdInputStream;
import org.eclipse.rdf4j.model.IRI;
import org.eclipse.rdf4j.query.BindingSet;
import org.eclipse.rdf4j.repository.RepositoryConnection;
import org.eclipse.rdf4j.repository.sparql.SPARQLRepository;
import org.eclipse.rdf4j.rio.RDFFormat;
import org.wikidata.wdtk.datamodel.implementation.PropertyIdValueImpl;
import org.wikidata.wdtk.datamodel.interfaces.PropertyIdValue;
import org.wikidata.wdtk.datamodel.interfaces.Sites;
import org.wikidata.wdtk.dumpfiles.DumpProcessingController;
import org.wikidata.wdtk.dumpfiles.EntityTimerProcessor;
import org.wikidata.wdtk.dumpfiles.MwLocalDumpFile;
import org.wikidata.wdtk.rdf.PropertyRegister;
import org.wikidata.wdtk.rdf.RdfSerializer;
import picocli.CommandLine;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;

import static org.wikidata.wdtk.datamodel.helpers.Datamodel.SITE_WIKIDATA;

class ZstdDumpFile extends MwLocalDumpFile {
    ZstdDumpFile(String filepath) {
        super(filepath);
    }

    @Override
    public InputStream getDumpFileStream() throws IOException {
        if (!this.getPath().toString().contains("zstd")) {
            return super.getDumpFileStream();
        }
        return new ZstdInputStream(new BufferedInputStream(Files.newInputStream(this.getPath(), StandardOpenOption.READ)));
    }
}

public class Main {
    @CommandLine.Option(names = { "--entity-filter" })
    Path[] entityFilterPaths;

    @CommandLine.Parameters(paramLabel = "FILE", description = "Wikidata JSON dump file to process")
    Path dumpFile;

    private static PropertyRegister wdPropertyRegisterFromSparql() {
        final PropertyRegister propertyRegister = PropertyRegister.getWikidataPropertyRegister();

        final SPARQLRepository repository = new SPARQLRepository("https://query.wikidata.org/sparql");
        repository.initialize();
        final RepositoryConnection connection = repository.getConnection();

        var query = connection.prepareTupleQuery("SELECT ?prop ?type WHERE { ?prop wikibase:propertyType ?type }");
        try (final var result = query.evaluate()) {
            while (result.hasNext()) {
                final BindingSet solution = result.next();
                final IRI property = (IRI)solution.getValue("prop");
                final IRI propType = (IRI)solution.getValue("type");
                final PropertyIdValue propId = new PropertyIdValueImpl(property.getLocalName(), SITE_WIKIDATA);
                propertyRegister.setPropertyType(propId, propType.toString());

            }
            return propertyRegister;
        } finally {
            repository.shutDown();
        }
    }

    private void run() throws IOException {
        final DumpProcessingController dumpProcessingController = new DumpProcessingController("wikidatawiki");
        final Sites sites = dumpProcessingController.getSitesInformation();
        final PropertyRegister propertyRegister = wdPropertyRegisterFromSparql();

        final ZstdDumpFile dumpFile = new ZstdDumpFile(this.dumpFile.toString());
        RdfSerializer serializer = new RdfSerializer(RDFFormat.NTRIPLES, OutputStream.nullOutputStream(), sites, propertyRegister);
        serializer.setTasks(RdfSerializer.TASK_SIMPLE_STATEMENTS | RdfSerializer.TASK_ITEMS);

        // Run serialization
        dumpProcessingController.setOfflineMode(true);
        final EntityTimerProcessor timerProcessor = new EntityTimerProcessor(0);
        serializer.open();
        timerProcessor.open();
        dumpProcessingController.registerEntityDocumentProcessor(serializer, null, true);
        dumpProcessingController.registerEntityDocumentProcessor(timerProcessor, null, true);
        dumpProcessingController.processDump(dumpFile);
        serializer.close();
        timerProcessor.close();
    }

    public static void main(String[] args) throws IOException {
        final Main main = new Main();
        new CommandLine(main).parseArgs(args);
        main.run();
    }
}
