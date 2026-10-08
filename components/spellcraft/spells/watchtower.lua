-- The watchtower's brain: is anyone standing in its shadow? Every call and loop turn costs mana (spellcraft).
return {
	decide = function(ctx)
		local o = ctx.observation
		return o ~= nil and #o.bodies > 0
	end,
}
