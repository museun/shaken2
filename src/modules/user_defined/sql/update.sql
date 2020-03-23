UPDATE 
	user_commands 
SET
	name = ?,
	body = ?,
	uses = ?,
	disabled = ?
WHERE 
	name = ?
