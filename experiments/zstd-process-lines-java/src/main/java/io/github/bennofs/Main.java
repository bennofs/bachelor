package io.github.bennofs;

import com.github.luben.zstd.ZstdInputStream;
import picocli.CommandLine;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.Scanner;
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
            final BufferedReader scanner = new BufferedReader(new InputStreamReader(inputStream));
            int itemCount = 0;
            long startNano = System.nanoTime();
            while (true) {
                final String line = scanner.readLine();
                if (line == null) break;
                itemCount += 1;

                if ((itemCount % 100000) == 0) {
                    long nowNano = System.nanoTime();
                    double itemsPerSec = itemCount / ((nowNano - startNano) / 1000000000L);
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
