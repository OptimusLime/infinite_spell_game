-- What the editor is before any part is installed: a toolbar, an empty stage that says how to fill it and a
-- status bar (Claude floats over it all, see chat.luau). Parts (editor-* packages) replace the toolbar and status bar when installed.
local ui = require("ui")
local kit = require("kit")

local shell = {}

local function installed_parts()
	local n = 0
	for _, p in (store.get("project.packages") or {}).components or {} do
		if string.match(p.name, "^editor%-") and p.name ~= "editor-panels" then n += 1 end
	end
	return n
end

-- The editor's own packages (an empty engine has these and nothing else).
local OWN = { layout = true, ["node-graph"] = true, ["shader-forge"] = true, ["ui-kit"] = true }

-- Packages installed beyond the editor's own, apps included: what the status bar counts while no panel is.
local function installed_packages()
	local packages = store.get("project.packages") or {}
	local n = 0
	for _, list in { packages.components or {}, packages.apps or {} } do
		for _, p in list do
			if not OWN[p.name] then n += 1 end
		end
	end
	return n
end

-- The status bar's count, in plain words: panels once the editor has them, else the packages so far.
function shell.installed_label(panels: number, packages: number): string
	if panels > 0 then return `{panels} editor panels installed` end
	if packages == 1 then return "1 package installed" end
	if packages > 1 then return `{packages} packages installed` end
	return "Nothing installed yet"
end

function shell.toolbar()
	local c = kit.c
	local logo = kit.bound(ui.node({ id = "logo", style = { width = kit.u(18), height = kit.u(18), radius = kit.u(5), background = c.accent, align = "center", justify = "center" } }, {
		ui.node({ style = { width = kit.u(6), height = kit.u(6), radius = kit.u(3), background = c.on_accent } }),
	}), kit.color_path("accent"), "fill")
	local recipe = store.get("editor.ui.recipe")
	return kit.bound(ui.row({ id = "panel:toolbar", style = { width = "100%", height = kit.bars().toolbar, shrink = 0, padding_x = kit.u(12), gap = kit.u(12), align = "center", background = c.header } }, {
		ui.row({ style = { gap = kit.u(8), align = "center", shrink = 0 } }, { logo, kit.text("Atelico", { bold = true, size = kit.m.font_size + 1 }) }),
		kit.rule(true),
		kit.text(if recipe and recipe ~= "" then string.match(recipe, "^(%S+)") else "New project", { tone = "muted" }),
		kit.spacer(),
	}), kit.color_path("header"), "fill")
end

function shell.empty()
	local c = kit.c
	return ui.column({ id = "panel:empty", style = { width = "100%", grow = 1, align = "center", justify = "center", gap = kit.u(10), background = c.viewport } }, {
		kit.text("An empty editor", { bold = true, size = kit.m.font_size + 6 }),
		kit.text("Everything here is a package: open the Store, find panels by what they do, install them.", { tone = "muted" }),
		kit.text("Or apply a recipe (a set of packages and a layout): app recipe apply app-editor", { tone = "muted", mono = true, size = kit.m.small + 1 }),
	})
end

function shell.status()
	local c = kit.c
	local h = kit.bars().status
	local caption = store.get("runtime.caption") or ""
	local bar = ui.row({ id = "panel:status", style = { width = "100%", height = h, shrink = 0, padding_x = kit.u(12), gap = kit.u(14), align = "center", background = c.header } }, {
		kit.text(shell.installed_label(installed_parts(), installed_packages()), { bold = true, size = kit.m.small + 1, color = c.muted, style = { shrink = 0 } }),
		kit.spacer(),
		if caption ~= "" then kit.text(caption, { id = "caption", bold = true, size = kit.m.small + 1, color = c.accent }) else nil,
	})
	return kit.bound(bar, kit.color_path("header"), "fill")
end

return shell
