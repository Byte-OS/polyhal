# Bootstrapping from 32-bit with the Multiboot specification.
# See https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

# Build the 2M Table Mapper.
.macro Page2MTable base
    .set    n, \base
    .rept   512
        .quad n + 0x83
    .set    n, n + 0x200000
    .endr
.endm

.macro MapAll2MPage name, addr
\name:
    Page2MTable  \addr
.endm

.macro Page1GHugeTable before, until
    .set    n, \before * 0x40000000
    .rept   \until - \before
        .quad n + 0x83
    .set    n, n + 0x40000000
    .endr
.endm

.section .multiboot
.balign 4
.type multiboot_header, STT_OBJECT
multiboot_header:
    .int    {mb_hdr_magic}                      # magic: 0x1BADB002
    .int    {mb_hdr_flags}                      # flags
    .int    -({mb_hdr_magic} + {mb_hdr_flags})  # checksum
    .int    multiboot_header - {kernel_offset}  # header_addr
    .int    _skernel - {kernel_offset}          # load_addr
    .int    _load_end - {kernel_offset}         # load_end
    .int    _end - {kernel_offset}              # bss_end_addr
    .int    _start - {kernel_offset}            # entry_addr
    .int    {graphic}                           # graphic mode
    .int    0                                   # graphic pixel width
    .int    0                                   # graphic pixel height
    .int    32                                  # graphic pixel color bits

# Common code in 32-bit, prepare states to enter 64-bit.
.code32
.global _start
.global _secondary_start
_start:
    mov     edi, eax                            # arg1: magic: 0x2BADB002
    mov     esi, ebx                            # arg2: multiboot info

    lgdt    [.Ltmp_gdt_desc - {kernel_offset}]         # load the temporary GDT
    # set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set PAE, PGE bit in CR4
    mov     eax, cr4
    or      eax, {cr4}
    mov     cr4, eax

    # load the temporary page table
    lea     eax, [_boot_page_table - {kernel_offset}]
    mov     cr3, eax

    # set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    rdmsr
    or      eax, {efer}
    wrmsr

    # set protected mode, write protect, paging bit in CR0
    mov     eax, cr0
    or      eax, {cr0}
    mov     cr0, eax

    ljmp    0x10, offset bsp_entry64 - {kernel_offset}    # 0x10 is code64 segment

_secondary_start:
     # set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set PAE, PGE bit in CR4
    mov     eax, {cr4}
    mov     cr4, eax

    # load the temporary page table
    lea     eax, [_boot_page_table - {kernel_offset}]
    mov     cr3, eax

    # set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    mov     edx, 0
    mov     eax, {efer}
    wrmsr

    # set protected mode, write protect, paging bit in CR0
    mov     eax, {cr0}
    mov     cr0, eax
    ljmp    0x10, offset ap_entry64 - {kernel_offset}     # 0x10 is code64 segment

.code64
bsp_entry64:
    # clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set RSP to boot stack
    movabs  rsp, offset bstack_top

    # call rust_entry(magic, mbi)
    movabs  rax, offset {entry}
    call    rax
    jmp     .Lhlt

ap_entry64:
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set RSP to high address (already set in ap_start.S)
    mov     rax, {kernel_offset}
    add     rsp, rax

    # call rust_entry_secondary(magic)
    movabs  rax, offset {entry_secondary}
    call    rax
    jmp     .Lhlt
.Lhlt:
    hlt
    jmp     .Lhlt

.section .rodata
.balign 8
.Ltmp_gdt_desc:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   # limit
    .long   .Ltmp_gdt - {kernel_offset}            # base

.section .data
.balign 16
.Ltmp_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00af9b000000ffff    # 0x10: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x18: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
.Ltmp_gdt_end:

.balign 4096
.global _boot_page_table
_boot_page_table:
    # 0x0000_0000 ~ 0xffff_ffff
    .quad _boot_mapping_pdpt - {kernel_offset} + 0x3   # PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 255
    .quad _boot_mapping_pdpt - {kernel_offset} + 0x3   # PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 255

.balign 4096
.global _boot_mapping_pdpt
# FIXME: may not work on macOS using hvf as the CPU does not support 1GB page (pdpe1gb)
_boot_mapping_pdpt:
    .quad .Lboot_pd1 - {kernel_offset} + 0x3 # PRESENT | WRITABLE | paddr(0x0)
    .quad .Lboot_pd2 - {kernel_offset} + 0x3 # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .quad .Lboot_pd3 - {kernel_offset} + 0x3 # PRESENT | WRITABLE | HUGE_PAGE | paddr(0x8000_0000)
    .quad .Lboot_pd4 - {kernel_offset} + 0x3 # PRESENT | WRITABLE | HUGE_PAGE | paddr(0xc000_0000)
    Page1GHugeTable 4, 512

.balign 4096
MapAll2MPage  .Lboot_pd1, 0x00000000
MapAll2MPage  .Lboot_pd2, 0x40000000
MapAll2MPage  .Lboot_pd3, 0x80000000
MapAll2MPage  .Lboot_pd4, 0xc0000000
