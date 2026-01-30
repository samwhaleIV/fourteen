namespace WAM.Core.Internal.Generator {
    internal sealed class UniqueGuidGenerator:IGenerator<string> {
        private readonly HashSet<string> _uuids = [];
        public string Next() {
            string uuid;
            do {
                uuid = Guid.NewGuid().ToString();
            } while(_uuids.Contains(uuid));
            _uuids.Add(uuid);
            return uuid;
        }

        public void Reset() {
            _uuids.Clear();
        }
    }
}
