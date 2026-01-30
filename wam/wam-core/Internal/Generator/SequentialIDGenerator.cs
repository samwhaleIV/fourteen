namespace WAM.Core.Internal.Generator {
    internal sealed class SequentialIDGenerator:IGenerator<int> {
        private int _id = 0;
        public int Next() {
            var id = _id;
            _id += 1;
            return id;
        }

        public void Reset() {
            _id = 0;
        }
    }
}
