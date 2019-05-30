package com.github.bennofs;

import com.github.luben.zstd.ZstdInputStream;
import org.wikidata.wdtk.datamodel.interfaces.EntityDocumentProcessor;
import org.wikidata.wdtk.datamodel.interfaces.ItemDocument;
import org.wikidata.wdtk.datamodel.interfaces.LexemeDocument;
import org.wikidata.wdtk.datamodel.interfaces.PropertyDocument;
import org.wikidata.wdtk.dumpfiles.DumpProcessingController;
import org.wikidata.wdtk.dumpfiles.EntityTimerProcessor;
import org.wikidata.wdtk.dumpfiles.MwLocalDumpFile;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;

class ZstdDumpFile extends MwLocalDumpFile {
    public ZstdDumpFile(String filepath) {
        super(filepath);
    }

    @Override
    public InputStream getDumpFileStream() throws IOException {
        return new ZstdInputStream(new BufferedInputStream(Files.newInputStream(this.getPath(), StandardOpenOption.READ)));
    }
}

public class Main {
    private static final String DUMP_FILE = "/run/media/bennofs/alpha/wikidata-20190506.json.zstd1";

    public static void main(String[] args) throws IOException {
	    final DumpProcessingController dumpProcessingController = new DumpProcessingController("wikidatawiki");
	    dumpProcessingController.setOfflineMode(true);
	    dumpProcessingController.registerEntityDocumentProcessor(new EntityTimerProcessor(0), null, true);
        dumpProcessingController.registerEntityDocumentProcessor(new EntityDocumentProcessor() {
            @Override
            public void processItemDocument(ItemDocument itemDocument) {
                itemDocument.toString();
            }

            @Override
            public void processPropertyDocument(PropertyDocument propertyDocument) {
                propertyDocument.toString();
            }

            @Override
            public void processLexemeDocument(LexemeDocument lexemeDocument) {
                lexemeDocument.toString();

            }
        }, null, true);

	    final ZstdDumpFile dumpFile = new ZstdDumpFile(DUMP_FILE);
	    dumpProcessingController.processDump(dumpFile);
    }
}
