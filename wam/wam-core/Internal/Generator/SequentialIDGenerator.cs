namespace WAM.Core.Internal.Generator {
    internal sealed class SequentialIDGenerator:IGenerator<uint> {
        private uint _id = 0;
        public uint Next() {
            var id = _id;
            _id += 1;
            return id;
        }

        public void Reset() {
            _id = 0;
        }
    }
}
