namespace WAM.CLI {

    readonly record struct Command(
        Action<IEnumerable<string>> Action,
        string Description
    );

    internal class Program {
        static bool FirstTime = true;
        static bool FromCommandLine = false;

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

        static readonly Dictionary<string,Command> Commands = new() {
            {
                "argstest", new Command {
                    Action = ArgsTest,
                    Description = "print list of 'args' that were supplied with the command (for debugging this cli)"
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
                    Description = "invert the colors of the console output"
                }
            }
        };

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
            var counter = 1;
            foreach(var command in Commands) {
                Console.WriteLine($"{counter}. {command.Key}: {command.Value.Description}");
                counter += 1;
            }
        }

        static void ArgsTest(IEnumerable<string> args) {
            Console.WriteLine($"args: {string.Join(", ",args)}");
        }

        static void TexturePack() {

        }
    }
}
