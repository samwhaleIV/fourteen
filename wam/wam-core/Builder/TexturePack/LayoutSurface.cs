using SkiaSharp;

namespace WAM.Core.Builder.TexturePack {
    internal sealed class LayoutSurface(int size,int id):IDisposable {

        private const int LOSSLESS_ENCODING = 101;

        private readonly CollisionValues[,] collisionMap = new CollisionValues[size,size];
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

        private enum CollisionValues {
            Nothing = 0,
            Texture = 1,
            TransparentBuffer = 2
        }

        private readonly record struct AreaLocation(int X,int Y,bool IsEdge);

        private static IEnumerable<AreaLocation> Iterate(Area area) {
            var endX = area.X + area.Width;
            var endY = area.Y + area.Height;
            for(var y = area.Y;y < endY;y++) {
                for(var x = area.X;x < endX;x++) {

                    bool isEdge =
                        x <= area.X ||
                        x >= endX - 1 ||
                        y <= area.Y ||
                        y >= endY - 1;

                    yield return new AreaLocation(x,y,isEdge);
                }
            }
        }

        private void FillCollisionMap(Area area,PackPadding padding) {
            foreach(var location in Iterate(area)) {
                var value = CollisionValues.Texture;
                if(location.IsEdge && padding == PackPadding.TransparentBuffer) {
                    value = CollisionValues.TransparentBuffer;
                }
                collisionMap[location.X,location.Y] = value;
            }
        }

        private bool AreaFits(Area area,PackPadding padding) {
            foreach(var location in Iterate(area)) {
                var value = collisionMap[location.X,location.Y];
                if(value == CollisionValues.Nothing) {
                    continue;
                }
                if(
                    value == CollisionValues.TransparentBuffer &&
                    padding == PackPadding.TransparentBuffer &&
                    location.IsEdge           
                ) {
                    continue;
                }
                return false;
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

        private void DrawBitmap(SKBitmap bitmap,int x,int y,PackPadding padding) {
            switch(padding) {
                case PackPadding.None:
                    surface.Canvas.DrawBitmap(bitmap,x,y);
                    break;
                case PackPadding.EdgeExtension:
                    DrawEdgeExtended(bitmap,x,y);
                    break;
                case PackPadding.TransparentBuffer:
                    surface.Canvas.DrawBitmap(bitmap,x + 1,y + 1);
                    break;
                default:
                    throw new NotImplementedException();
            }
        }

        // TODO: Add use granular pack packing if desired. The code already supports varying between modes
        public bool TryAddBitmap(SKBitmap bitmap,PackPadding padding,out Area area) {
            var (width,height) = (bitmap.Width,bitmap.Height);

            if(padding != PackPadding.None) {
                width += 2;
                height += 2;
            }

            area = new Area();
            foreach(var location in Iterate(new() {
                X = 0,
                Y = 0,
                Width = Size - width,
                Height = Size - height
            })) {
                area = new Area(location.X,location.Y,width,height);
                if(AreaFits(area,padding)) {
                    FillCollisionMap(area,padding);
                    DrawBitmap(bitmap,area.X,area.Y,padding);
                    return true;
                }
            }
            return false;
        }

        private void DrawEdgeExtended(SKBitmap bitmap,int x,int y) {
            var (w,h) = (bitmap.Width,bitmap.Height);
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,0,w,h),       new SKRect(x+1,y+1,x+w+1,y+h+1)); //Center
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,0,1,h),       new SKRect(x,y+1,x+1,y+h+1)); //Left Column
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,0,w,1),       new SKRect(x+1,y,x+w+1,y+1)); //Top Row
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(w-1,0,w,h),     new SKRect(x+w+1,y+1,x+w+2,y+h+1)); //Right Column
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,h-1,w,h),     new SKRect(x+1,y+h+1,x+1+w,y+h+2)); //Bottom Row
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,0,1,1),       new SKRect(x,y,x+1,y+1)); //Top Left
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(0,h-1,1,h),     new SKRect(x,y+h+1,x+1,y+h+2)); //Bottom Left
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(w-1,0,w,1),     new SKRect(x+w+1,y,x+w+2,y+1)); //Top Right
            surface.Canvas.DrawBitmap(bitmap,   new SKRect(w-1,h-1,w,h),   new SKRect(x+w+1,y+h+1,x+w+2,y+h+2)); //Bottom Right
        }
    }
}
