///usr/bin/env jbang "$0" "$@" ; exit $?
//JAVA 17+
//DEPS com.fasterxml.jackson.core:jackson-databind:2.18.2
//DEPS info.picocli:picocli:4.7.6

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;

import javax.tools.JavaCompiler;
import javax.tools.ToolProvider;
import java.io.*;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Command(
    name = "ser",
    mixinStandardHelpOptions = true,
    version = "1.3",
    description = "Loads built-in or external .java/.class files, deserializes JSON into the target class, and outputs Java-serialized Hex."
)
public class Ser implements Callable<Integer> {

    @Option(
        names = {"-c", "--class-file"},
        description = "Optional target .java or .class file(s). Can be specified multiple times. Omit when targeting built-in JDK classes."
    )
    private List<File> targetFiles;

    @Option(
        names = {"-n", "--class-name"},
        description = "Fully qualified class name (FQCN) of the root target class (e.g., java.lang.String or com.example.User). Required if -c is omitted."
    )
    private String explicitClassName;

    @Option(
        names = {"-j", "--json"},
        required = true,
        description = "Raw JSON string, or path to a JSON file prefixed with '@' (e.g., @data.json)."
    )
    private String jsonInput;

    public static void main(String[] args) {
        int exitCode = new CommandLine(new Ser()).execute(args);
        System.exit(exitCode);
    }

    @Override
    public Integer call() throws Exception {
        boolean hasFiles = targetFiles != null && !targetFiles.isEmpty();

        if (!hasFiles && (explicitClassName == null || explicitClassName.isBlank())) {
            System.err.println("Error: Either --class-file (-c) or --class-name (-n) must be specified.");
            return 1;
        }

        if (hasFiles) {
            for (File file : targetFiles) {
                if (!file.exists()) {
                    System.err.println("Error: Target file not found -> " + file.getAbsolutePath());
                    return 1;
                }
            }
        }

        // 1. Load target Class (either from external files or built-in/existing classpath)
        Class<?> rootTargetClass = hasFiles
                ? loadFromExternalFiles(targetFiles, explicitClassName)
                : Class.forName(explicitClassName);

        if (!Serializable.class.isAssignableFrom(rootTargetClass)) {
            System.err.println("Warning: Class [" + rootTargetClass.getName() + "] does not implement java.io.Serializable. Serialization will fail.");
        }

        // 2. Read JSON payload (either inline string or @file path)
        String rawJson = jsonInput.startsWith("@")
                ? Files.readString(Path.of(jsonInput.substring(1)))
                : jsonInput;

        // 3. Deserialize JSON into the target class instance
        ObjectMapper mapper = new ObjectMapper();
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        Object instance = mapper.readValue(rawJson, rootTargetClass);

        // 4. Serialize Java object and encode as Hex
        String hexOutput = serializeToHex((Serializable) instance);
        System.out.println(hexOutput);

        return 0;
    }

    private Class<?> loadFromExternalFiles(List<File> files, String rootClassNameOverride) throws Exception {
        List<String> javaSourcePaths = new ArrayList<>();
        Set<URL> classpathUrls = new LinkedHashSet<>();

        Path tempCompileDir = Files.createTempDirectory("jbang_dyn_compile_");
        tempCompileDir.toFile().deleteOnExit();

        for (File file : files) {
            Path path = file.toPath().toAbsolutePath();
            String fileName = path.getFileName().toString();

            if (fileName.endsWith(".java")) {
                javaSourcePaths.add(path.toString());
            } else if (fileName.endsWith(".class")) {
                if (path.getParent() != null) {
                    classpathUrls.add(path.getParent().toUri().toURL());
                }
            } else {
                throw new IllegalArgumentException("Unsupported file type (only .java and .class are supported): " + fileName);
            }
        }

        // If explicit FQCN with package is provided for a .class file, resolve its package root directory
        File primaryFile = files.get(0);
        if (primaryFile.getName().endsWith(".class") && rootClassNameOverride != null && rootClassNameOverride.contains(".")) {
            String[] pkgParts = rootClassNameOverride.split("\\.");
            Path classRoot = primaryFile.toPath().toAbsolutePath().getParent();
            for (int i = 0; i < pkgParts.length - 1; i++) {
                if (classRoot != null) {
                    classRoot = classRoot.getParent();
                }
            }
            if (classRoot != null) {
                classpathUrls.add(classRoot.toUri().toURL());
            }
        }

        // Compile all .java files together in a single batch so cross-references resolve properly
        if (!javaSourcePaths.isEmpty()) {
            JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
            if (compiler == null) {
                throw new IllegalStateException("System Java compiler not found. Ensure JBang is running with a full JDK (not a JRE).");
            }

            List<String> compilerArgs = new ArrayList<>();
            compilerArgs.add("-d");
            compilerArgs.add(tempCompileDir.toAbsolutePath().toString());
            compilerArgs.addAll(javaSourcePaths);

            int compileResult = compiler.run(null, null, null, compilerArgs.toArray(new String[0]));
            if (compileResult != 0) {
                throw new RuntimeException("Failed to compile one or more external .java files.");
            }
            classpathUrls.add(tempCompileDir.toUri().toURL());
        }

        // Determine the FQCN of the primary (first) file
        String rootFqcn = (rootClassNameOverride != null)
                ? rootClassNameOverride
                : inferClassName(primaryFile.toPath());

        URLClassLoader classLoader = new URLClassLoader(
                classpathUrls.toArray(new URL[0]),
                this.getClass().getClassLoader()
        );

        return Class.forName(rootFqcn, true, classLoader);
    }

    private String inferClassName(Path file) throws IOException {
        String fileName = file.getFileName().toString();
        if (fileName.endsWith(".java")) {
            String source = Files.readString(file);
            String simpleClassName = fileName.substring(0, fileName.length() - 5);
            Matcher matcher = Pattern.compile("^\\s*package\\s+([a-zA-Z0-9_.]+)\\s*;", Pattern.MULTILINE).matcher(source);
            if (matcher.find()) {
                return matcher.group(1) + "." + simpleClassName;
            }
            return simpleClassName;
        } else if (fileName.endsWith(".class")) {
            return fileName.substring(0, fileName.length() - 6);
        }
        throw new IllegalArgumentException("Cannot infer class name from file: " + fileName);
    }

    private String serializeToHex(Serializable obj) throws IOException {
        try (ByteArrayOutputStream baos = new ByteArrayOutputStream();
             ObjectOutputStream oos = new ObjectOutputStream(baos)) {
            oos.writeObject(obj);
            oos.flush();
            return HexFormat.of().formatHex(baos.toByteArray());
        }
    }
}