import json
import random

def load_map(map_entries):
    entries = json.loads(map_entries)
    entities = []

    banned_hammer = "hammer" + str(random.choice([1, 2]))
    banned_enemies = banned_enemy_groups()

    for entry in entries:
        obj = basic_components(entry)
        classes = entry["classes"]

        should_make = True

        for group in banned_enemies:
            if group in classes:
                should_make = False
        
        if banned_hammer in classes:
            should_make = False
        
        if "item" in classes:
            obj.append(Item("Weapon"))

        if "boss" in classes:
            obj += [
                Alignment("Enemies"),
                Chaser(Alignment("Players"), distance = 5),
                Hurtbox(hp = 2, knockback = 2.7),
                Hitbox([1, 0.7]),
                Speed(0.095),
                Health(max = 4, current = 4),
            ]

        elif "enemy" in classes:
            obj += [
                Alignment("Enemies"),
                Chaser(Alignment("Players"), distance = 5),
                Hurtbox(hp = 1, knockback = 2.7),
                Hitbox([1, 0.7]),
                Speed(0.075),
                Health(max = 4, current = 4),
            ]

        if should_make:
            entities.append(obj)

    return entities

def banned_enemy_groups():
    seed = random.randint(1, 10)
    groups = []

    # 40% of the time, ban group 1
    if seed <= 4:
        groups.append("enemygroup1")
    
    # other 40%: ban group 2
    elif seed <= 8:
        groups.append("enemygroup2")

    # otherwise neither group is banned;
    # player must deal with both

    return groups
