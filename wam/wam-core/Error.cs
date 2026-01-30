namespace WAM.Core {
    public readonly struct Error {
        public required string Message { get; init; }
        public static Error Create(string message) {
            return new Error {
                Message = message
            };
        }
    }
}
