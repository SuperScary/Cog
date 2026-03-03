use std::path::Path;

///
pub fn detect_language_from_path(file_path: &str) -> String {
    let path = Path::new(file_path);

    let extension = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext.to_lowercase(),
        None => return "Plain Text".to_string(),
    };

    let ext = extension.as_str();

    programming_language(ext)
        .or_else(|| config(ext))
        .or_else(|| shell(ext))
        .or_else(|| web_framework_and_templating(ext))
        .or_else(|| markup_and_documentation(ext))
        .or_else(|| build_and_devops(ext))
        .or_else(|| database(ext))
        .or_else(|| data_and_serialization(ext))
        .or_else(|| shader_and_gpu(ext))
        .or_else(|| game_development(ext))
        .or_else(|| graphics_3d(ext))
        .or_else(|| audio(ext))
        .or_else(|| video(ext))
        .or_else(|| archive(ext))
        .or_else(|| security_and_certificate(ext))
        .or_else(|| font(ext))
        .or_else(|| ai_and_ml(ext))
        .or_else(|| low_level(ext))
        .or_else(|| miscellaneous(ext))
        .map(|s| s.to_string())
        .unwrap_or(extension)
}

fn programming_language(ext: &str) -> Option<&'static str> {
    Some(match ext {
        // Blockchain
        "cairo" => "Cairo",
        "clar" => "Clarity",
        "move" => "Move",
        "sol" => "Solidity",
        "vy" => "Vyper",

        // Cloth
        "clib" => "Cloth Library",
        "co" | "cloth" => "Cloth",

        // Esoteric
        "b" => "B Language",
        "bf" => "Brainfuck",
        "golf" => "GolfScript",
        "lol" => "LOLCODE",
        "malbolge" => "Malbolge",
        "whitespace" => "Whitespace",

        // Functional / Academic
        "agda" => "Agda",
        "ats" => "ATS",
        "elm" => "Elm",
        "erl" => "Erlang",
        "ex" | "exs" => "Elixir",
        "fs" | "fsi" | "fsx" => "F#",
        "hs" => "Haskell",
        "idris" | "idris2" => "Idris",
        "ml" => "OCaml",
        "mli" => "OCaml Interface",
        "mlton" => "MLton",
        "opa" => "Opa",
        "purs" => "PureScript",
        "reason" => "ReasonML",
        "res" => "ReScript",
        "rkt" => "Racket",
        "sml" => "Standard ML",
        "ur" => "Ur/Web",

        // Game Scripting
        "gd" => "GDScript",
        "gml" => "GameMaker Language",
        "hscript" => "HScript",
        "mcfunction" => "Minecraft Function",
        "nut" => "Squirrel",
        "renpy" => "Ren'Py",
        "sk" => "Skript",

        // Hardware / HDL
        "bsv" => "Bluespec",
        "mcrl2" => "mCRL2",
        "pde" => "Processing",
        "spin" => "Promela",
        "sv" => "SystemVerilog",
        "svh" => "SystemVerilog Header",
        "vhdl" | "vhd" => "VHDL",

        // JVM
        "bsh" => "BeanShell",
        "ceylon" => "Ceylon",
        "clj" | "cljs" | "cljc" => "Clojure",
        "frege" => "Frege",
        "golo" => "Golo",
        "groovy" | "gvy" | "gy" | "gsh" => "Groovy",
        "java" => "Java",
        "kt" | "ktm" => "Kotlin",
        "kts" => "Kotlin Script",
        "scala" => "Scala",

        // Legacy / Classic
        "ada" | "adb" | "ads" => "Ada",
        "cob" | "cbl" => "COBOL",
        "f90" | "f95" | "f03" | "f08" | "for" => "Fortran",
        "forth" | "fth" => "Forth",
        "foxpro" => "Visual FoxPro",
        "modula" => "Modula-2",
        "pas" | "pp" => "Pascal",
        "pl1" => "PL/I",
        "smalltalk" | "st" => "Smalltalk",

        // Lisp Family
        "el" => "Emacs Lisp",
        "lisp" => "Lisp",
        "scm" => "Scheme",

        // Mobile
        "dart" => "Dart",
        "m" => "Objective-C",
        "mm" => "Objective-C++",
        "swift" => "Swift",

        // Modern / Emerging
        "carbon" => "Carbon",
        "chapel" => "Chapel",
        "gleam" => "Gleam",
        "grain" => "Grain",
        "hack" => "Hack",
        "janus" => "Janus",
        "koka" => "Koka",
        "mojo" => "Mojo",
        "pony" => "Pony",
        "red" => "Red",
        "ring" => "Ring",
        "roc" => "Roc",
        "vale" => "Vale",
        "vlang" => "V",
        "wren" => "Wren",

        // .NET
        "cs" => "C#",
        "vb" => "Visual Basic",

        // Scientific / Simulation
        "apl" | "dyalog" => "APL",
        "gams" | "gms" => "GAMS",
        "jl" => "Julia",
        "mat" => "MATLAB",
        "modelica" => "Modelica",
        "netlogo" => "NetLogo",
        "q" => "Q Language",
        "r" => "R",
        "sas" => "SAS",
        "simula" => "Simula",
        "stata" => "Stata",

        // Scripting
        "ahk" => "AutoHotkey",
        "au3" => "AutoIt",
        "awk" => "AWK",
        "lua" => "Lua",
        "moon" => "MoonScript",
        "php" => "PHP",
        "py" => "Python",
        "rb" => "Ruby",
        "tcl" => "Tcl",
        "vbs" => "VBScript",

        // Systems / Native
        "asm" | "s" => "Assembly",
        "beef" => "Beef",
        "c" => "C",
        "c3" => "C3",
        "cpp" | "cc" | "cxx" => "C++",
        "cr" => "Crystal",
        "cy" => "Cython",
        "d" => "D",
        "go" => "Go",
        "h" => "C Header",
        "hare" => "Hare",
        "hpp" | "hh" | "hxx" => "C++ Header",
        "hx" => "Haxe",
        "jai" => "Jai",
        "kit" => "Kit",
        "nim" => "Nim",
        "odin" => "Odin",
        "rs" => "Rust",
        "terra" => "Terra",
        "v" => "V",
        "vala" => "Vala",
        "zig" => "Zig",

        // Web
        "css" => "CSS",
        "html" | "htm" => "HTML",
        "js" => "JavaScript",
        "jsx" => "JavaScript React",
        "ts" => "TypeScript",
        "tsx" => "TypeScript React",

        _ => return None,
    })
}

fn config(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "babelrc" => "Babel Config",
        "cfg" => "Config",
        "conf" => "Config",
        "editorconfig" => "EditorConfig",
        "env" => "Environment",
        "eslintrc" => "ESLint Config",
        "ini" => "INI",
        "json" => "JSON",
        "kdl" => "KDL",
        "npmrc" => "NPM Config",
        "pnpmfile" => "PNPM Config",
        "prettierrc" => "Prettier Config",
        "properties" => "Properties",
        "toml" => "TOML",
        "xml" => "XML",
        "yaml" | "yml" => "YAML",
        "yarnrc" => "Yarn Config",
        _ => return None,
    })
}

fn shell(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "bash" => "Bash",
        "bat" | "cmd" => "Batch",
        "csh" => "C Shell",
        "fish" => "Fish",
        "ion" => "Ion Shell",
        "ksh" => "KornShell",
        "ps1" => "PowerShell",
        "psm1" => "PowerShell Module",
        "sh" => "Shell",
        "tcsh" => "TC Shell",
        "zsh" => "Zsh",
        _ => return None,
    })
}

fn web_framework_and_templating(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "astro" => "Astro",
        "ejs" => "Embedded JavaScript",
        "haml" => "Haml",
        "handlebars" => "Handlebars",
        "htmx" => "HTMX",
        "jsp" => "Java Server Pages",
        "jspx" => "Java Server Pages XML",
        "less" => "Less",
        "liquid" => "Liquid",
        "mustache" => "Mustache",
        "pug" => "Pug",
        "sass" => "Sass",
        "scss" => "SCSS",
        "svelte" => "Svelte",
        "twig" => "Twig",
        "vue" => "Vue",
        _ => return None,
    })
}

fn markup_and_documentation(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "adoc" => "AsciiDoc",
        "bib" => "BibTeX",
        "md" => "Markdown",
        "org" => "Org Mode",
        "pod" => "Perl POD",
        "rst" => "reStructuredText",
        "tex" => "LaTeX",
        "txt" => "Plain Text",
        _ => return None,
    })
}

fn build_and_devops(ext: &str) -> Option<&'static str> {
    Some(match ext {
        // Build Systems
        "bazel" | "bzl" => "Bazel",
        "build" => "Buck Build",
        "cabal" => "Cabal",
        "cmake" => "CMake",
        "csproj" | "fsproj" => "MSBuild",
        "gradle" => "Gradle",
        "make" | "mk" => "Makefile",
        "meson" => "Meson",
        "mod" => "Go Module",
        "ninja" => "Ninja Build",
        "nix" => "Nix",
        "pom" => "Maven POM",
        "props" => "MSBuild Props",
        "sln" => "Visual Studio Solution",
        "sum" => "Go Checksum",
        "targets" => "MSBuild Targets",
        "vbproj" => "VB Project",
        "zon" => "Zig Build",

        // CI/CD
        "azure-pipelines" => "Azure Pipelines",
        "circleci" => "CircleCI",
        "github" => "GitHub Actions",
        "gitlab-ci" => "GitLab CI",
        "travis" => "Travis CI",

        // Containers / Cloud
        "chart" => "Helm Chart",
        "dockerfile" => "Dockerfile",
        "dockerignore" => "Docker Ignore",
        "hcl" => "HCL",
        "helm" => "Helm",
        "k8s" => "Kubernetes",
        "nomad" => "Nomad",
        "tf" => "Terraform",
        "vault" => "Vault Config",

        // Package / OS
        "apkbuild" => "Alpine APKBUILD",
        "ebuild" => "Gentoo Ebuild",
        "snap" => "Snapcraft",

        // Systemd
        "mount" => "Systemd Mount",
        "service" => "Systemd Service",
        "socket" => "Systemd Socket",
        "timer" => "Systemd Timer",

        _ => return None,
    })
}

fn database(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "db" => "Database",
        "dbml" => "Database Markup Language",
        "graphql" | "gql" => "GraphQL",
        "mysql" => "MySQL",
        "psql" => "PostgreSQL",
        "sql" => "SQL",
        "sqlite" => "SQLite",
        _ => return None,
    })
}

fn data_and_serialization(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "csv" => "CSV",
        "ipynb" => "Jupyter Notebook",
        "lock" => "Lockfile",
        "lockb" => "Binary Lockfile",
        "plist" => "Property List",
        "proto" => "Protocol Buffers",
        "tsv" => "TSV",
        _ => return None,
    })
}

fn shader_and_gpu(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "glsl" | "vert" | "frag" => "GLSL",
        "hlsl" => "HLSL",
        "shader" => "ShaderLab",
        "wgsl" => "WGSL",
        _ => return None,
    })
}

fn game_development(ext: &str) -> Option<&'static str> {
    Some(match ext {
        // Godot
        "godot" => "Godot",
        "tres" => "Godot Resource",

        // Unity
        "anim" => "Unity Animation",
        "meta" => "Unity Meta",
        "prefab" => "Unity Prefab",
        "unity" => "Unity",

        // Unreal
        "umap" => "Unreal Map",
        "uplugin" => "Unreal Plugin",
        "uproject" => "Unreal Project",

        _ => return None,
    })
}

fn graphics_3d(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "blend" => "Blender",
        "dae" => "Collada",
        "fbx" => "FBX",
        "glb" => "GLB",
        "gltf" => "GLTF",
        "mtl" => "Material",
        "obj" => "Wavefront OBJ",
        _ => return None,
    })
}

fn audio(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "flac" => "FLAC",
        "mid" | "midi" => "MIDI",
        "mp3" => "MP3",
        "ogg" => "OGG",
        "wav" => "WAV",
        _ => return None,
    })
}

fn video(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "mkv" => "MKV",
        "mov" => "MOV",
        "mp4" => "MP4",
        "webm" => "WebM",
        _ => return None,
    })
}

fn archive(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "7z" => "7-Zip",
        "ear" => "Enterprise Archive",
        "gz" => "GZip",
        "rar" => "RAR",
        "tar" => "TAR Archive",
        "war" => "Web Archive",
        "zip" => "ZIP Archive",
        _ => return None,
    })
}

fn security_and_certificate(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "cer" => "Certificate",
        "crt" => "Certificate",
        "csr" => "Certificate Signing Request",
        "key" => "Private Key",
        "pem" => "PEM Certificate",
        "pfx" => "PKCS#12",
        _ => return None,
    })
}

fn font(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "otf" => "OpenType Font",
        "ttf" => "TrueType Font",
        "woff" => "WOFF",
        "woff2" => "WOFF2",
        _ => return None,
    })
}

fn ai_and_ml(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "ckpt" => "Checkpoint",
        "onnx" => "ONNX Model",
        "pb" => "TensorFlow Model",
        "pt" => "PyTorch Model",
        _ => return None,
    })
}

fn low_level(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "bin" => "Binary",
        "elf" => "ELF Binary",
        "hex" => "Intel HEX",
        "img" => "Disk Image",
        "ld" => "Linker Script",
        "map" => "Linker Map",
        _ => return None,
    })
}

fn miscellaneous(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "desktop" => "Desktop Entry",
        "diff" => "Diff",
        "gitattributes" => "Git Attributes",
        "gitignore" => "Git Ignore",
        "ics" => "iCalendar",
        "log" => "Log File",
        "patch" => "Patch",
        "torrent" => "BitTorrent",
        "vcf" => "vCard",
        _ => return None,
    })
}
