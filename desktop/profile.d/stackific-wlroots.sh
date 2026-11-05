# Let wlroots compositors (MangoWM, noctalia-greeter's) render in software when no GPU driver works: in VMs without 3D,
# or on a GPU whose driver is missing. wlroots refuses Mesa's llvmpipe otherwise, and MangoWM, whose renderer is
# GLES-only, then exits. greetd sources /etc/profile, so this reaches every session it starts.
export WLR_RENDERER_ALLOW_SOFTWARE=1
