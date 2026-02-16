using WAM.Core;
using WAM.Core.Builder;

namespace WAM.CLI {

    readonly record struct Command(
        Action<IEnumerable<string>> Action,
        string Description,
        bool Hidden = false
    );

    internal class Program {
        static bool FirstTime = true;
        static bool FromCommandLine = false;

        private enum BuildingCommandParameter {
            Source,
            Destination,
            TargetNamespace,
            ManifestName,
        }

        static readonly Dictionary<string,Command> Commands = new() {
            {
                "argstest", new Command {
                    Action = ArgsTest,
                    Hidden = true,
                    Description = "print list of 'args' that were supplied with the command"
                }
            },
            {
                "help", new Command {
                    Action = Help,
                    Description = "get a list of command names and their descriptions (but you already know that)"
                }
            },
            {
                "invert", new Command {
                    Action = InvertColors,
                    Hidden = true,
                    Description = "invert the colors of the console output"
                }
            },
            {
                "build-manifest", new Command {
                    Action = CreateManifestPackage,
                    Description = "create a wam manifest: -i <src> -o <dest> -n <target namespace> (-m <manifest file name>) (-guid)",
                    Hidden = false
                }
            },
            {
                "crash", new Command {
                    Action = static _ => {
                        throw new Exception("🤣 Did you know you can put emojis in exceptions? 😲");
                    },
                    Description = "the quickest way to the bottom is to jump",
                    Hidden = true
                }
            }
        };

        static void Execute(string[] args) {
            if(args.Length > 0) {
                var command = args[0].ToLower();
                if(!string.IsNullOrWhiteSpace(command)) {
                    if(Commands.TryGetValue(command,out var function)) {
                        function.Action.Invoke(new ArraySegment<string>(args,1,args.Length-1));
                    } else {
                        Console.WriteLine($"unknown command: {command}");
                    }
                } else {
                    Console.WriteLine("null command");
                }
            } else {
                Console.WriteLine("null command");
            }
            Main([]);
        }

        static string[] FilterArgs(string[] args) {
            for(var i = 0;i<args.Length;i++) {
                args[i] = args[i].Trim(' ','"','\'');
            }
            return args;
        }

        static void Main(string[] args) {
            if(FromCommandLine) {
                return;
            }

            if(FirstTime) {
                Console.ForegroundColor = ConsoleColor.White;
                Console.BackgroundColor = ConsoleColor.Black;
            }

            if(args.Length > 0) {
                FromCommandLine = true;
                Execute(FilterArgs(args));
                return;
            }

            if(FirstTime) {
                Console.WriteLine("welcome to the wimpy asset manager, aka, wam");
                Console.WriteLine("i can help get your best assets where you want them to go ;)");
                FirstTime = false;
            }

            Console.Write("enter command (or 'help' for a list of commands): ");
            var input = Console.ReadLine()?.Split(' ') ?? [];
            Execute(FilterArgs(input));
        }

        static void InvertColors(IEnumerable<string> args) {
            if(Console.ForegroundColor == ConsoleColor.White) {
                Console.BackgroundColor = ConsoleColor.White;
                Console.ForegroundColor = ConsoleColor.Black;
                Console.WriteLine("ping");
            } else {
                Console.BackgroundColor = ConsoleColor.Black;
                Console.ForegroundColor = ConsoleColor.White;
                Console.WriteLine("pong");
            }
        }

        static void Help(IEnumerable<string> args) {
            var showHidden = false;
            foreach(var item in args) {
                if(item == "hidden") {
                    showHidden = true;
                    break;
                }
            }
            var counter = 1;
            var hiddenCount = 0;
            foreach(var command in Commands) {
                if(!showHidden && command.Value.Hidden) {
                    hiddenCount += 1;
                    continue;
                }
                string description = command.Value.Description;
                if(string.IsNullOrWhiteSpace(description)) {
                    description = "<no description>";
                }
                Console.WriteLine($"{counter}. {command.Key}{(command.Value.Hidden ? "*" : "")}: {description}");
                counter += 1;
            }

            Console.WriteLine(showHidden ?
                "* = a hidden/debug command" :
                $"(use 'help hidden' to see {hiddenCount} hidden command{(hiddenCount == 1 ? "" : "s")})"
            );
        }

        static void ArgsTest(IEnumerable<string> args) {
            Console.WriteLine($"args: {string.Join(", ",args)}");
        }

        static bool ShouldProceed(string prompt) {
            if(FromCommandLine) {
                return true;
            }
            Console.WriteLine(prompt);
            Console.Write("are you sure you want to proceed? write 'yes' to proceed: ");
            var result = Console.ReadLine();
            if(result == null) {
                return false;
            }
            result = result.Trim().ToLower();
            if(string.IsNullOrWhiteSpace(result)) {
                return false;
            }
            return result == "yes";
        }

        static void ClearDirectory(string directory) {
            if(!Directory.Exists(directory)) {
                return;
            }
            foreach(var file in Directory.GetFiles(directory,"*",SearchOption.TopDirectoryOnly)) {
                File.Delete(file);
                Console.WriteLine($"deleted file '{file}'");
            }
            foreach(var subdirectory in Directory.GetDirectories(directory,"*",SearchOption.TopDirectoryOnly)) {
                Directory.Delete(subdirectory,true);
                Console.WriteLine($"deleted directory '{subdirectory}'");
            }
        }

        static void QualifyDirectory(string filePath) {
            var directory = Path.GetDirectoryName(filePath);
            if(string.IsNullOrEmpty(directory)) {
                return;
            }
            if(Directory.Exists(directory)) {
                return;
            }
            Directory.CreateDirectory(directory);
        }

        static void CreateManifestPackage(IEnumerable<string> args) {
            BuildingCommandParameter? parameter = null;

            string? src = null;
            string? dst = null;
            string? targetNamespace = null;
            string? manifestName = null;
            bool guid = false;

            //Description = "create a wam manifest. \"build-manifest -in <src> -out <dest> -ns <namespace> (-name <*.json>) (-guid)\"",
            foreach(var arg in args) {
                if(parameter != null) {
                    switch(parameter.Value) {
                        case BuildingCommandParameter.Source:
                            if(src != null) {
                                Console.WriteLine("warning: multiple source parameters in argument stream");
                            }
                            src = arg;
                            break;
                        case BuildingCommandParameter.Destination:
                            if(dst != null) {
                                Console.WriteLine("warning: multiple destination parameters in argument stream");
                            }
                            dst = arg;
                            break;
                        case BuildingCommandParameter.TargetNamespace:
                            if(targetNamespace != null) {
                                Console.WriteLine("warning: multiple target namespace parameters in argument stream");
                            }
                            targetNamespace = arg;
                            break;
                        case BuildingCommandParameter.ManifestName:
                            if(manifestName != null) {
                                Console.WriteLine("warning: multiple manifest name parameters in argument stream");
                            }
                            manifestName = arg;
                            break;
                    }
                    parameter = null;
                    continue;
                }
                
                if(!arg.StartsWith('-')) {
                    Console.WriteLine($"unexpected token '{arg}'");
                    return;
                }
                var parameterString = arg.TrimStart('-').ToLowerInvariant();
                switch(parameterString) {
                    case "i":
                    case "src":
                    case "in":
                    case "input":
                    case "source":
                        parameter = BuildingCommandParameter.Source;
                        break;
                    case "o":
                    case "dst":
                    case "out":
                    case "output":
                    case "destination":
                        parameter = BuildingCommandParameter.Destination;
                        break;
                    case "n":
                    case "ns":
                    case "namespace":
                        parameter = BuildingCommandParameter.TargetNamespace;
                        break;
                    case "m":
                    case "manifest":
                        parameter = BuildingCommandParameter.ManifestName;
                        break;
                    case "guid":
                        if(guid == true) {
                            Console.WriteLine("warning: multiple guid flags in argument stream");
                        }
                        guid = true;
                        break;
                    default:
                        Console.WriteLine($"unknown parameter '{parameterString}'");
                        return;
                }
            }

            if(parameter != null) {
                Console.WriteLine("unexpected end of parameter sequence");
                return;
            }

            if(src == null) {
                Console.WriteLine("no source specified (use \"-i <src>\")");
                return;
            }

            if(dst == null) {
                Console.WriteLine("no destination specified (use \"-o <dst>\")");
                return;
            }

            if(targetNamespace == null) {
                Console.WriteLine("no target namespace specified (use \"-n <namespace>\")");
                return;
            }

            var settings = new WamManifestSettings(src,dst,targetNamespace,null,guid,false,manifestName!);
            if(
                !FromCommandLine &&
                Directory.Exists(settings.Destination) &&
                Directory.GetFileSystemEntries(settings.Destination,"*",SearchOption.AllDirectories).Length > 0
            ) {
                if(!ShouldProceed(
                    "the output directory already has files and this action will clear them"
                )) {
                    Console.WriteLine("action aborted");
                    return;
                }
            }

            var manifest = new WamManifest(settings);
            var error = manifest.Build();
            if(error.HasValue) {
                Console.WriteLine(error.Value.Message);
                return;
            }

            var json = manifest.GetJson(); /* Generate the JSON before fucking with any files in case it fails */

            ClearDirectory(settings.Destination);

            foreach(var generatedFile in manifest.GetGeneratedFiles()) {
                var destination = generatedFile.Destination;
                QualifyDirectory(destination);
                File.WriteAllBytes(destination,generatedFile.Data);
                Console.WriteLine($"created output file '{Path.GetRelativePath(settings.Destination,destination)}' (size: {generatedFile.Data.Length})");
            }

            foreach(var mappedFile in manifest.GetFileMaps()) {
                var destination = mappedFile.Destination;
                QualifyDirectory(destination);
                File.Copy(mappedFile.Source,destination,true);
                Console.WriteLine($"copied file to output '{Path.GetRelativePath(settings.Destination,destination)}' from '{mappedFile.Source}'");
            }

            var manifestPath = Path.Combine(settings.Destination,settings.ManifestOutputFile);
            File.WriteAllText(manifestPath,json);
            Console.WriteLine($"created manifest file at '{manifestPath}'");
        }
    }
}
