-- #rrggbb <-> hue (degrees), saturation, value (0..1).
local color = {}

function color.rgb(hex)
	local h = hex:gsub("#", "")
	return tonumber(h:sub(1, 2), 16) / 255, tonumber(h:sub(3, 4), 16) / 255, tonumber(h:sub(5, 6), 16) / 255
end

function color.hex(r, g, b)
	local byte = function(v) return math.clamp(math.floor(v * 255 + 0.5), 0, 255) end
	return string.format("#%02x%02x%02x", byte(r), byte(g), byte(b))
end

function color.hsv(hex)
	local r, g, b = color.rgb(hex)
	local max, min = math.max(r, g, b), math.min(r, g, b)
	local d = max - min
	local h = 0
	if d > 0 then
		if max == r then h = ((g - b) / d) % 6
		elseif max == g then h = (b - r) / d + 2
		else h = (r - g) / d + 4 end
	end
	return h * 60, if max > 0 then d / max else 0, max
end

function color.from_hsv(h, s, v)
	local c = v * s
	local x = c * (1 - math.abs((h / 60) % 2 - 1))
	local m = v - c
	local k = math.floor(h / 60) % 6
	local r, g, b = 0, 0, 0
	if k == 0 then r, g, b = c, x, 0
	elseif k == 1 then r, g, b = x, c, 0
	elseif k == 2 then r, g, b = 0, c, x
	elseif k == 3 then r, g, b = 0, x, c
	elseif k == 4 then r, g, b = x, 0, c
	else r, g, b = c, 0, x end
	return color.hex(r + m, g + m, b + m)
end

function color.valid(hex)
	return type(hex) == "string" and hex:match("^#?%x%x%x%x%x%x$") ~= nil
end

return color
