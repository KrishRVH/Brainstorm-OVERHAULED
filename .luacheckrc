-- Balatro globals
globals = {
    "Brainstorm",
    "G",
    "STR_PACK",
    "STR_UNPACK",
    "Controller",
    "Event",
    "Game",
    "Particles",
    "UIBox",
    "UIBox_button",
    "attention_text",
    "compress_and_save",
    "copy_table",
    "create_option_cycle",
    "create_text_input",
    "create_toggle",
    "darken",
    "get_compressed",
    "lighten",
    "number_format",
    "play_sound",
    "random_string"
}

-- LOVE runtime
read_globals = {
    "love"
}

-- Performance: cache all globals
cache = true

-- Allow trailing whitespace (stylua handles this)
ignore = {
    "611", -- trailing whitespace
    "612", -- trailing whitespace in string
    "613", -- trailing whitespace in comment
    "614", -- trailing whitespace in empty line
}

-- Cap ordinary code without rejecting required FFI strings or metadata comments.
max_line_length = false
max_code_line_length = 120
max_string_line_length = false
max_comment_line_length = false

-- Cyclomatic complexity threshold
max_cyclomatic_complexity = 30

-- Balatro callbacks include intentionally unused parameters.
unused_args = false
unused_secondaries = false
self = false

-- Exclude the untracked game source.
exclude_files = {
    "BalatroSource/**"
}

-- Allow certain patterns
allow_defined_top = true
