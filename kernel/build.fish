#!/usr/bin/fish
begin
    set -l __ORIG (rg 'VERSION: &str = "(.*)"' --only-matching --no-line-number --no-filename)
    set -l __NEWV (echo $__ORIG | rg '\([0-9A-Fa-f ]+\)' --only-matching | rg '[0-9A-Fa-f]+' --only-matching | awk '{print(strtonum("0x" $1) + 1)}' | xargs -I {} printf '%0.6x' {})
    ruplacer 'VERSION: &str = "kernel ([0-9.]+) \(([0-9A-Fa-f ]+)\)"' "VERSION: &str = \"kernel \$1 ($__NEWV)\"" --go --quiet
    echo "KERNEL_VERSION: $__NEWV"
end
