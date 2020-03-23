SELECT 
	name,
	body,
	room,
	uses,
	owner,
	disabled,
	created_at
FROM 
	user_commands
where
	room = ?
