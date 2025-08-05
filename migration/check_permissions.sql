-- Check all resources
SELECT id, name, description FROM resource ORDER BY created_at;

-- Check all roles
SELECT id, name, level FROM role ORDER BY level DESC;

-- Check all permissions with their resources
SELECT p.id, p.action, r.name as resource_name, p.description 
FROM permission p 
JOIN resource r ON p.resource_id = r.id 
ORDER BY r.name, p.action;

-- Check role permissions for admin role
SELECT r.name as role_name, p.action, res.name as resource_name
FROM role_permission rp
JOIN role r ON rp.role_id = r.id
JOIN permission p ON rp.permission_id = p.id
JOIN resource res ON p.resource_id = res.id
WHERE r.name = 'admin'
ORDER BY res.name, p.action;

-- Check role permissions for super_admin role
SELECT r.name as role_name, p.action, res.name as resource_name
FROM role_permission rp
JOIN role r ON rp.role_id = r.id
JOIN permission p ON rp.permission_id = p.id
JOIN resource res ON p.resource_id = res.id
WHERE r.name = 'super_admin'
ORDER BY res.name, p.action;

-- Count permissions by role
SELECT r.name as role_name, COUNT(*) as permission_count
FROM role_permission rp
JOIN role r ON rp.role_id = r.id
GROUP BY r.name
ORDER BY permission_count DESC;