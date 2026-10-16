-- Part: the status bar (mode, scene, caption, MCP port).
local panels = require("panels")
return { name = "status", slot = "status", render = function() return panels.status() end }
