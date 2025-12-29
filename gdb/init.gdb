target remote :1234

# load rust pretty printers
source ./gdb/gdb_load_rust_pretty_printers.py

# for some reason, pwndbg's kernel-vmmap doesn't work
set kernel-vmmap none

# continue to kernel entry
tb kernel::kernel_entry
c

# put your custom gdb scripts in custom.gdb
source ./gdb/custom.gdb