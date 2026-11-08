# Shared Serena settings. Import this from any module that runs Serena.

# Pinned Serena (PyPI `serena-agent`, github.com/oraios/serena)
serena_version := "2.1.0"

# Serena's own home (config, logs, language servers, news state), instead of the shared ~/.serena, where another
# Serena version's state (such as its pickled news_read.pkl) stops the pinned one from starting
serena_home := justfile_directory() / ".tools" / "serena"
serena_env := "SERENA_HOME='" + serena_home + "'"
