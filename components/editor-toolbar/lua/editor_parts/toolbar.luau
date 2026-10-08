-- Part: the toolbar with scene tabs and Play/Stop (the play/stop runtime).
local panels = require("panels")
return { name = "toolbar", slot = "toolbar", render = function() return panels.toolbar() end }
