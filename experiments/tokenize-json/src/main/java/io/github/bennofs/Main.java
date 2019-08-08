package io.github.bennofs;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import com.github.luben.zstd.ZstdInputStream;
import picocli.CommandLine;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.zip.GZIPInputStream;

public class Main implements Runnable {
    @CommandLine.Parameters(paramLabel = "FILE", description = "Wikidata JSON dump file to process")
    Path dumpFile;

    public void run() {
        try {
            final InputStream inputStream;

            if (dumpFile.toString().contains("zstd")) {
                inputStream = new ZstdInputStream(new BufferedInputStream(Files.newInputStream(dumpFile, StandardOpenOption.READ)));
            } else {
                inputStream = new GZIPInputStream(new BufferedInputStream(Files.newInputStream(dumpFile, StandardOpenOption.READ)));
            }
            final JsonFactory jsonFactory = new JsonFactory();
            final JsonParser parser = jsonFactory.createParser(inputStream);
            int itemCount = 0;
            long startNano = System.nanoTime();
            long nesting = 0;
            while (true) {
                final JsonToken token = parser.nextToken();
                if (token == null) break;
                if (token.isStructStart()) {
                    nesting += 1;
                }
                if (token.isStructEnd()) {
                    nesting -= 1;
                }

                if (nesting == 1) {
                    itemCount += 1;
                }

                if ((itemCount % 10000) == 0 && nesting == 1) {
                    long nowNano = System.nanoTime();
                    if (nowNano - startNano == 0) continue;
                    double itemsPerSec = itemCount / ((nowNano - startNano) / 1000000000.0);
                    System.err.println("Processed " + itemCount + " items (" + (long)itemsPerSec + " items per second" + ")");
                }
            }

        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public static void main(String[] args) throws IOException {
        final Main main = new Main();
        new CommandLine(main).execute(args);
    }
}
