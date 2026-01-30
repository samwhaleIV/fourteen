namespace WAM.Core {
    public struct Result<T> where T:new() {

        private const string FALLBACK_ERROR = "unknown error";
        private const string VALUE_OKAY = "no error";

        private bool HasValue { get; init; }

        public string Error { get; private init; }
        public T Value { get; private init; }

        public static Result<T> Err() {
            return new() {
                Error = FALLBACK_ERROR,
                Value = new(),
                HasValue = false
            };
        }

        public static Result<T> Err(string? message) {
            if(string.IsNullOrWhiteSpace(message)) {
                message = FALLBACK_ERROR;
            }
            return new() {
                Error = message,
                Value = new(),
                HasValue = false
            };
        }

        public static Result<T> Ok(T manifest) {
            return new() {
                Error = VALUE_OKAY,
                Value = new(),
                HasValue = true,
            };
        }

        public bool IsErr {
            get => !HasValue;
        }

        public bool IsOk {
            get => HasValue;
        }
    }
}
