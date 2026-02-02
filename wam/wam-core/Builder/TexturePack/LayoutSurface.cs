using SkiaSharp;

namespace WAM.Core.Builder.TexturePack {
    internal sealed class LayoutSurface(int size,int id):IDisposable {

        private const int LOSSLESS_ENCODING = 101;

        private readonly byte[,] collisionMap = new byte[size,size];
        private readonly SKSurface surface = SKSurface.Create(new SKImageInfo(
            size,
            size,
            SKColorType.Rgba8888,
            SKAlphaType.Unpremul
        ));

        public int ID { get; init; } = id;
        public int Size { get; init; } = size;

        public void Dispose() {
            surface.Dispose();
        }

        private static IEnumerable<(int X,int Y)> Iterate(Area area) {
            var endX = area.X + area.Width;
            var endY = area.Y + area.Height;
            for(var y = area.Y;y < endY;y++) {
                for(var x = area.X;x < endX;x++) {
                    yield return (x,y);
                }
            }
        }

        private void FillCollisionMap(Area area) {
            foreach(var (x,y) in Iterate(area)) {
                collisionMap[x,y] = byte.MaxValue;
            }
        }

        private bool AreaFits(Area area) {
            foreach(var (x,y) in Iterate(area)) {
                if(collisionMap[x,y] > byte.MinValue) {
                    return false;
                }
            }
            return true;
        }

        public byte[] ExportPNG(ImageFormat imageFormat) {
            using var snapshot = surface.Snapshot();
            using var data = snapshot.Encode(
                imageFormat.ToSkFormat(),LOSSLESS_ENCODING
            );
            return data.ToArray();
        }

        public bool TryAddBitmap(SKBitmap bitmap,out Area area) {
            var (width,height) = (bitmap.Width,bitmap.Height);
            area = new Area();
            foreach(var (x,y) in Iterate(new() {
                X = 0,
                Y = 0,
                Width = Size - width,
                Height = Size - height
            })) {
                area = new Area(x,y,width,height);
                if(AreaFits(area)) {
                    FillCollisionMap(area);
                    surface.Canvas.DrawBitmap(bitmap,x,y);
                    return true;
                }
            }
            return false;
        }
    }
}
