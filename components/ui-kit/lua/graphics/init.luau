-- Resource descriptions are shared data. Rust owns loading and GPU allocation.
local graphics = {}
function graphics.texture(path_or_pixels) return path_or_pixels end
function graphics.shader(path_or_source) return path_or_source end
function graphics.material(props) return props end
-- A small reusable pixel-art helper; the image content is authored in Luau.
function graphics.pixels(rows, palette)
    local pixels = {}
    local width = #rows[1]
    for _, row in rows do
        assert(#row == width, "pixel rows must have the same width")
        for i = 1, width do table.insert(pixels, palette[string.sub(row, i, i)] or "00000000") end
    end
    return { width = width, height = #rows, pixels = pixels }
end
return graphics
