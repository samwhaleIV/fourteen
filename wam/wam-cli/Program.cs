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
                "debug", new Command {
                    Action = Debug,
                    Description = string.Empty,
                    Hidden = true
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
                Execute(args);
                return;
            }

            if(FirstTime) {
                Console.WriteLine("welcome to the wimpy asset manager, aka, wam");
                Console.WriteLine("i can help get your best assets where you want them to go ;)");
                FirstTime = false;
            }

            Console.Write("enter command (or 'help' for a list of commands): ");
            var input = Console.ReadLine()?.Split(' ') ?? [];
            Execute(input);
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
                Console.WriteLine($"deleted directory '{directory}'");
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

        static void CreateManifestPackage(WamManifestSettings settings) {
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

        static void Debug(IEnumerable<string> args) {
            var settings = new WamManifestSettings(
                Source: @"C:\Users\pinks\OneDrive\Documents\Rust Projects\fourteen\wam\wam-core\test-content\",
                Destination: @"C:\Users\pinks\OneDrive\Documents\Rust Projects\fourteen\wam\wam-core\test-content\debug-output",
                TargetNamespace: "alias"
            );
            CreateManifestPackage(settings);
        }
    }
}
