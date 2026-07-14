-- Balatro globals
globals = {
    "Brainstorm",
    "G",
    "SMODS",
    "STR_PACK",
    "STR_UNPACK",
    "Controller",
    "DynaText",
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
    "lovely",
    "nfs",
    "number_format",
    "play_sound",
    "pseudorandom",
    "pseudorandom_element",
    "pseudoseed",
    "random_string",
    "sendDebugMessage"
}

-- Standard library extensions and LuaJIT FFI
read_globals = {
    "bit",
    "ffi",
    "jit",
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
    "631", -- long FFI signatures and UI literals
}

-- Max line length (matching stylua config)
max_line_length = 120

-- Cyclomatic complexity threshold
max_cyclomatic_complexity = 30

-- Allow unused args with underscore prefix
unused_args = false
unused_secondaries = false
self = false

-- Exclude the untracked game source.
exclude_files = {
    "BalatroSource/**"
}

files["UI.lua"] = {
    -- UI code often has deeply nested callbacks
    max_cyclomatic_complexity = 30
}

-- Allow certain patterns
allow_defined_top = true
