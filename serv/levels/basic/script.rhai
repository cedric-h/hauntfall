let load_map = |map_entries| {
	for entry in map_entries {
		print entry["location"][0];
		entry["location"][0] = 0;
	}
	return map_entries;
};
