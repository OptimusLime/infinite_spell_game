-- Frames an app's render in the Portal page: a title bar over a scrolling, padded workspace.
local ui = require("ui")
local t = require("phone.theme")
local shell = {}
function shell.render(env)
	local wide = env.width >= 900
	local body = env.content or ui.node({})
	local fill = body.style and body.style.height == "100%"
	return ui.column({ id = "desktop", style = { width = "100%", height = "100%", background = t.colors.background } }, {
		ui.row({ id = "header", style = { width = "100%", height = 64, shrink = 0, padding_x = if wide then 28 else 16, align = "center", border = 0, background = t.colors.background } }, {
			t.text(env.app_name or "", 30, nil, { id = "title", style = { text_wrap = "none" } }),
		}),
		ui.node({ style = { width = "100%", height = 4, shrink = 0, background = t.colors.text } }),
		ui.column({ id = "scroll", style = { width = "100%", grow = 1, min_height = 0, padding = if wide then 28 else 16, gap = 20, scroll = not fill } }, { body }),
	})
end
return shell
