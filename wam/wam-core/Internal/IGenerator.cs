namespace WAM.Core.Internal {
    internal interface IGenerator<T> {
        public T Next();
        public void Reset();
    }
}
