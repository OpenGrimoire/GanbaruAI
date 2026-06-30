UPDATE projects
SET color = CASE id
    WHEN 'project-routine-eat' THEN 13
    WHEN 'project-routine-learning' THEN 8
    WHEN 'project-routine-reading' THEN 25
    WHEN 'project-routine-exercise' THEN 0
    WHEN 'project-routine-hygiene' THEN 15
    WHEN 'project-routine-social' THEN 21
    WHEN 'project-routine-chores' THEN 4
    WHEN 'project-routine-leisure' THEN 31
    WHEN 'project-routine-meditate' THEN 23
    WHEN 'project-routine-sleep' THEN 30
END
WHERE id IN (
    'project-routine-eat',
    'project-routine-learning',
    'project-routine-reading',
    'project-routine-exercise',
    'project-routine-hygiene',
    'project-routine-social',
    'project-routine-chores',
    'project-routine-leisure',
    'project-routine-meditate',
    'project-routine-sleep'
);
