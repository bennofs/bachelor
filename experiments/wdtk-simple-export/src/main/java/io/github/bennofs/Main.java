package io.github.bennofs;

import com.github.luben.zstd.ZstdInputStream;
import org.eclipse.rdf4j.rio.RDFFormat;
import org.wikidata.wdtk.datamodel.interfaces.*;
import org.wikidata.wdtk.dumpfiles.DumpProcessingController;
import org.wikidata.wdtk.dumpfiles.EntityTimerProcessor;
import org.wikidata.wdtk.dumpfiles.MwLocalDumpFile;
import org.wikidata.wdtk.rdf.PropertyRegister;
import org.wikidata.wdtk.rdf.RdfSerializer;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;

class ZstdDumpFile extends MwLocalDumpFile {
    ZstdDumpFile(String filepath) {
        super(filepath);
    }

    @Override
    public InputStream getDumpFileStream() throws IOException {
        return new ZstdInputStream(new BufferedInputStream(Files.newInputStream(this.getPath(), StandardOpenOption.READ)));
    }
}

public class Main {
    public static void main(String[] args) throws IOException {
        final DumpProcessingController dumpProcessingController = new DumpProcessingController("wikidatawiki");
        dumpProcessingController.setOfflineMode(true);
        dumpProcessingController.registerEntityDocumentProcessor(new EntityTimerProcessor(0), null, true);

        final Sites sites = dumpProcessingController.getSitesInformation();
        final PropertyRegister propertyRegister = PropertyRegister.getWikidataPropertyRegister();

        final ZstdDumpFile dumpFile = new ZstdDumpFile(args[0]);
        RdfSerializer serializer = new RdfSerializer(RDFFormat.NTRIPLES, OutputStream.nullOutputStream(), sites, propertyRegister);
        serializer.setTasks(RdfSerializer.TASK_ITEMS
                | RdfSerializer.TASK_SIMPLE_STATEMENTS);

        // Run serialization
        serializer.open();
        dumpProcessingController.registerEntityDocumentProcessor(serializer, null, true);
        dumpProcessingController.processDump(dumpFile);
        serializer.close();
    }
}