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

        static void Debug(IEnumerable<string> args) {
            var manifest = new WamManifest(WamManifestSettings.GetDefault(
                @"something\test-content\",
                @"something\test-output",
                @"alias"
            ));
            var error = manifest.Build();
            if(error.HasValue) {
                Console.WriteLine(error.Value.Message);
            }
            return;
        }
    }
}
