# Point noctalia-greeter at our asset folder (package stackific-desktop), its documented override: the greeter takes
# its logo from its assets and has no setting for another one. Our folder holds the Stackific OS logo and links the
# greeter's own fonts. greetd sources /etc/profile before the greeter, so the greeter sees it; nothing else reads it.
export NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets
