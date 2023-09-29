-- fields.star
    
import bits

type
    Field = (size: Int, data: Array<Byte>)
    Color = enum.of(Black, White)
    Index = (byte: Int, bit: Int)

def of(size: Int): Field =
    nCells = size * size
    nBytes = nCells / 8 + (nCells % 8 == 0 ? 0 : 1)
    Field(size, array.zeros(nBytes))

def Field.convertIndex(i: Int, j: Int): Index =
    flatIndex = i * it.size + j
    Index(flatIndex / 8, flatIndex % 8)

def Field.get(i: Int, j: Int): Color =
    index = it.convertIndex(i, j)

    it.data[index.byte].isBitSet(index.bit)
        ? Black : White

def Field.set(i: Int, j: Int, color: Color) =
    index = it.convertIndex(i, j)
    old   = it.data[index.byte]

    it.data[index.byte] = if color is
        White -> old.clearBit(index.bit)
        Black -> old.setBit(index.bit)

-- bmp.star
    
import fs

type
    ColorMapEntry = (r: Byte, g: Byte, b: Byte)
    RowOrder      = enum.of(TopDown, BottomUp)

-- https://www.fileformat.info/format/bmp/egff.htm

def cFileHeaderSize: Int = 14
def cDibHeaderSize:  Int = 40

def writeBmpHeader(file: File, colorMapSize: Int) =
    file.write(0x4d42us)  -- File type, always 4D42h ("BM")
    file.write(0ui)       -- Size of the file in bytes, 0 for uncompressed
    file.write(0us)       -- Always 0
    file.write(0us)       -- Always 0
    file.write〉-- Starting position of image data in bytes
        cFileHeaderSize + cDibHeaderSize + colorMapSize * 4

def writeDib3Header(
    file: File, width: Int, height: Int,
    rowOrder: RowOrder, dataSize: Int,
) =
    fixedHeight = if rowOrder is
        TopDown  ->  height
        BottomUp -> -height

    file.write(cDibHeaderSize) -- Size of this header in bytes
    file.write(width)          -- Image width in pixels
    file.write(fixedHeight)    -- Image height in pixels
    file.write(1us)            -- Number of color planes
    file.write(1us)            -- Number of bits per pixel
    file.write(0u)             -- Compression methods used
    file.write(dataSize)       -- Size of bitmap in bytes
    file.write(1000u)          -- Horizontal resolution in pixels per meter
    file.write(1000u)          -- Vertical resolution in pixels per meter
    file.write(2u)             -- Number of colors in the image
    file.write(0u)             -- Minimum number of important colors

def writeColorMapEntry(file: File, entry: ColorMapEntry) =
    file.write(entry.b)
    file.write(entry.g)
    file.write(entry.r)
    file.write(0u)

def write(
    file: File, width: Int, height: Int,
    colorMap: Array<ColorMapEntry>,
    rowOrder: RowOrder, data: Array<Byte>,
) =
    writeBmpHeader(file, colorMap.length)
    writeDib3Header(file, width, height, rowOrder, data.length)

    for entry in colorMap do
        writeColorMapEntry(file, entry)

    file.write(data)

-- main.star

import fs, fields, bmp

def main =
    field = fields.of(1024)
    x, y  = (512, 512)
    direction = Up

    go = f if direction is
        Left  -> x -= 1
        Up    -> y += 1
        Right -> x += 1
        Down  -> y -= 1

    while x >= 0 && y >= 0 &&
        x < field.size && y < field.size do

        if field[x, y] is
            White ->
                field[x, y] = Black
                direction = if direction is
                    Left  -> Up
                    Up    -> Right
                    Right -> Down
                    Down  -> Left

                go()
            Black ->
                field[x, y] = White
                direction = if direction is
                    Left  -> Down
                    Up    -> Left
                    Right -> Up
                    Down  -> Right

                go()

    file = fs.create('ant_path.bmp')

    bmp.write〉
        file, field.size, field.size,
        .colorMap = 〉
            (0, 255, 0), -- green background
            (0, 0, 255), -- blue path
        BottomUp, field.data,
