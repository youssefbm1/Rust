#! /usr/bin/python
#

counters = {}
symbols = {}
code = []
data = []

MEMORY_SIZE = 4096
NREGS = 16

CALLEE_SAVE = list(range(8))
CALLER_SAVE = list(range(8, 16))

IP = 0
ZERO = 1
SP = 2
TRASH = 3

BUSY_REGS = [IP, ZERO, SP, TRASH]


def find_reg(busy_regs):
    for i in range(NREGS):
        if i not in busy_regs:
            return i, busy_regs + [i]


def make_symbol(base="L"):
    counters[base] = counters.setdefault(base, 0) + 1
    return "{}_{}".format(base, counters[base])


def assign_here(symbol):
    symbols[symbol] = len(code)


def make_symbol_and_jump(busy_regs, base="L", cond_reg=None):
    symbol = make_symbol(base)
    if cond_reg is not None:
        jump_if(busy_regs, symbol, cond_reg)
    else:
        jump(symbol)
    return symbol


def move_if(target, source, cond):
    code.extend([1, target, source, cond])


def move(target, source):
    move_if(target, source, IP)


def store(target, source):
    code.extend([2, target, source])


def load(target, source):
    code.extend([3, target, source])


def loadimm(target, value):
    if type(value) == str:
        l, h = "low:{}".format(value), "high:{}".format(value)
        code.extend([4, target, l, h])
    elif value >= 2**15 or value < -2**15:
        if value < 0:
            value = 2**32 + value
        label = make_symbol("large_integer")
        data.append((label, [value & 0xff, (value >> 8) &
                             0xff, (value >> 16) & 0xff, (value >> 24) & 0xff]))
        loadimm(TRASH, label)
        load(target, TRASH)
    else:
        if value < 0:
            value = value + 65536
        l, h = value % 256, value // 256
        code.extend([4, target, l, h])


def sub(target, op1, op2):
    code.extend([5, target, op1, op2])


def add(busy_regs, target, op1, op2):
    r, _ = find_reg(busy_regs)
    sub(r, 1, op2)
    sub(target, op1, r)


def out(reg):
    code.extend([6, reg])


def out_number(reg):
    code.extend([8, reg])


def exit():
    code.append(7)


def jump_if(busy_regs, target, cond):
    t, busy_regs = make_reg(busy_regs, target)
    c, _ = make_reg(busy_regs, cond)
    move_if(0, t, c)


def jump(target):
    if type(target) != int:
        loadimm(IP, target)
    else:
        move(IP, target)


def make_reg(busy_regs, value):
    if type(value) == int:
        return value, busy_regs
    r, busy_regs = find_reg(busy_regs)
    loadimm(r, value)
    return r, busy_regs


def make_trash(value):
    if type(value) == int:
        return value
    loadimm(TRASH, value)
    return TRASH


def if_then_else(busy_regs, cond, then_code, else_code=None):
    if type(cond) == int:
        cond_reg = cond
    elif cond[0] == "ne":
        cond_reg, _ = find_reg(busy_regs)
        sub(cond_reg, cond[1], cond[2])
    elif cond[0] == "neconst":
        cond_reg, _ = find_reg(busy_regs)
        loadimm(cond_reg, cond[2])
        sub(cond_reg, cond[1], cond_reg)
    elif cond[0].startswith("eq"):
        return if_then_else(busy_regs, ("ne" + cond[0][2:], cond[1], cond[2]), else_code, then_code)
    else:
        raise Exception("Unknown condition {}".format(cond))

    then_label = make_symbol_and_jump(
        busy_regs + [cond_reg], "ite_then", cond_reg)

    # Else part
    if else_code:
        else_code(busy_regs)
    end_label = make_symbol_and_jump(busy_regs, "ite_end")

    # Then part
    assign_here(then_label)
    if then_code:
        then_code(busy_regs)

    # End
    assign_here(end_label)


def start_function(label):
    assign_here(label)


def end_function():
    pop(IP)


def jsr(label):
    ret = make_symbol("return_from_{}".format(label))
    push(ret)
    jump(label)
    assign_here(ret)


def push(value):
    loadimm(TRASH, 4)
    sub(SP, SP, TRASH)
    r = make_trash(value)
    store(SP, r)


def pop(into_reg):
    assert(into_reg != TRASH)
    loadimm(TRASH, -4)
    sub(SP, SP, TRASH)
    loadimm(TRASH, 4)
    sub(TRASH, SP, TRASH)
    load(into_reg, TRASH)


def string(s):
    label = make_symbol("str")
    data.append((label, s))
    return (label, len(s))


def do_print(s):
    name, len = string(s)
    push(10)
    push(11)
    loadimm(10, name)
    loadimm(11, len)
    jsr("print")
    pop(11)
    pop(10)


def append_data():
    for (label, content) in data:
        symbols[label] = len(code)
        code.extend(content)


def replace_labels():
    for (i, c) in enumerate(code):
        if type(c) == str:
            if c.startswith("low:"):
                code[i] = symbols[c[4:]] % 256
            elif c.startswith("high:"):
                code[i] = symbols[c[5:]] // 256
            else:
                code[i] = symbols[c]


def disassemble(fd):
    rev = {}
    for (k, v) in symbols.items():
        rev.setdefault(v, []).append(k)
    i = 0
    while i < len(code):
        if i in rev:
            for s in sorted(rev[i]):
                fd.write("{}:\n".format(s))
        fd.write("  {:04d} ".format(i))
        c = code[i:i+4]
        if c[0] == 1:
            fd.write("  move r{} <- r{} if r{} != 0".format(c[1], c[2], c[3]))
            i += 4
        elif c[0] == 2:
            fd.write("  store [r{}] <- r{}".format(c[1], c[2]))
            i += 3
        elif c[0] == 3:
            fd.write("  load r{} <- [r{}]".format(c[1], c[2]))
            i += 3
        elif c[0] == 4:
            fd.write(
                "  loadimm r{} <- #{}".format(c[1], load_imm_decode(c[2], c[3])))
            i += 4
        elif c[0] == 5:
            fd.write("  sub r{} <- r{} - r{}".format(c[1], c[2], c[3]))
            i += 4
        elif c[0] == 6:
            fd.write("  out r{}".format(c[1]))
            i += 2
        elif c[0] == 7:
            fd.write("  exit")
            i += 1
        elif c[0] == 8:
            fd.write("  out_number r{}".format(c[1]))
            i += 2
        else:
            fd.write("  ???")
            i += 1
        fd.write("\n")
    for (l, d) in data:
        fd.write("{}:\n".format(l))
        fd.write("  ???? {}\n".format(d))


def load_imm_decode(l, h):
    if type(l) == str:
        return l[4:]
    v = l | (h << 8)
    return v - 65536 if v & 0x8000 else v


def print_test():
    hello_addr, hello_len = string(b"Hello, world!\n")
    loadimm(10, hello_addr)
    loadimm(11, hello_len)
    jsr("print")
    happy_addr, happy_len = string(b"I am happy to be here\n")
    loadimm(10, happy_addr)
    loadimm(11, happy_len)
    jsr("print")
    exit()
    add_print_function()


def add_afact_function():
    if "afact" in symbols:
        return
    add_mult_function()
    # fact r10, result into data, uses r11, r12, r13 and r14
    start_function("afact")
    data.append(("acc", [0, 0, 0, 0]))
    loadimm(TRASH, "acc")
    loadimm(11, 1)
    store(TRASH, 11)
    assign_here("afact_loop")

    def do_mult(_):
        loadimm(TRASH, "acc")
        load(11, TRASH)
        move(12, 10)
        jsr("mult")
        loadimm(TRASH, "acc")
        store(TRASH, 11)
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        jump("afact_loop")
    if_then_else(CALLEE_SAVE + [10, 11], ("neconst", 10, 1), do_mult)
    end_function()


def fact_test():
    jsr("fact")
    exit()
    add_fact_function()


def afact_test():
    jsr("afact")
    exit()
    add_afact_function()


def rfact_test():
    jsr("rfact")
    exit()
    add_rfact_function()


def rfact_tr_test():
    jsr("rfact_tr")
    exit()
    add_rfact_tr_function()


def add_fact_function():
    if "fact" in symbols:
        return
    add_mult_function()
    # fact r10, result into r11, uses r12, r13 and r14
    start_function("fact")
    loadimm(11, 1)
    assign_here("fact_loop")

    def do_mult(_):
        move(12, 10)
        jsr("mult")
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        jump("fact_loop")
    if_then_else(CALLEE_SAVE + [10, 11], ("neconst", 10, 1), do_mult)
    end_function()


def add_rfact_function():
    if "rfact" in symbols:
        return
    add_mult_function()
    # fact r10, result into r11
    start_function("rfact")

    def do_recurse(_):
        push(10)
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        jsr("rfact")
        pop(12)
        jsr("mult")

    def return_1(_):
        loadimm(11, 1)
    if_then_else(CALLEE_SAVE + [10], ("eqconst", 10, 1), return_1, do_recurse)
    end_function()


def add_rfact_tr_function():
    if "rfact_tr" in symbols:
        return
    add_mult_function()
    # fact r10, result into r11
    start_function("rfact_tr")

    def do_recurse(_):
        push(10)
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        jsr("rfact_tr")
        pop(12)
        jump("mult")
    if_then_else(CALLEE_SAVE + [10], ("neconst", 10, 1), do_recurse)
    loadimm(11, 1)
    end_function()


def fibo_test():
    jsr("fibo")
    exit()
    add_fibo_function()


def add_fibo_function():
    if "fibo" in symbols:
        return
    # fibo r10, result into r11
    start_function("fibo")

    def do_recurse(_):
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        push(10)
        jsr("fibo")
        pop(10)
        push(11)
        loadimm(TRASH, 1)
        sub(10, 10, TRASH)
        jsr("fibo")
        pop(10)
        sub(11, ZERO, 11)
        sub(11, 10, 11)

    def return_0(_):
        loadimm(11, 0)
        end_function()

    def return_1(_):
        loadimm(11, 1)
        end_function()

    if_then_else(CALLEE_SAVE + [10], 10, None, return_0)
    if_then_else(CALLEE_SAVE + [10], ("eqconst", 10, 1), return_1, do_recurse)
    end_function()


def print_result(busy_regs, cond, s):
    def ps(b):
        def f(_):
            r = b"success" if b else b"failure"
            addr, len = string(s + b": " + r + b"\n")
            loadimm(10, addr)
            loadimm(11, len)
            jsr("print")
            if not r:
                exit()
        return f
    if_then_else(busy_regs, cond, ps(True), ps(False))


def mult_test():
    jsr("mult")
    exit()
    add_mult_function()


def add_mult_function():
    if "mult" in symbols:
        return
    # mult r11 x r12 into r11. Uses r13 (== -r11) and r14 (== r12) as intermediaries.
    start_function("mult")
    sub(13, ZERO, 11)
    move(14, 12)

    def do_add(_):
        sub(11, 11, 13)
        loadimm(TRASH, 1)
        sub(14, 14, TRASH)
        jump("mult_loop")
    assign_here("mult_loop")
    if_then_else(CALLEE_SAVE + [10, 11, 12, 13, 14],
                 ("neconst", 14, 1), do_add)
    end_function()


def add_print_function():
    if "print" in symbols:
        return
    # Print, string starts at r10, length is in r11
    start_function("print")
    loop = make_symbol("print_loop")
    assign_here(loop)

    def print_char(_):
        load(TRASH, 10)
        out(TRASH)
        loadimm(TRASH, -1)
        sub(10, 10, TRASH)
        loadimm(TRASH, 1)
        sub(11, 11, TRASH)
        jump(loop)
    if_then_else(CALLEE_SAVE + [10, 11], 11, print_char)
    end_function()


def jump_to_function_test():
    jsr("myfunc")
    exit()
    start_function("myfunc")
    loadimm(10, 42)
    end_function()


def push_pop_test():
    push(0)
    push(0)
    pop(1)
    pop(2)
    exit()


def hello_world_example():
    do_print(b"Hello, world!\n")
    exit()
    add_print_function()


def count_example():
    do_print(b"I will count from 1 to 10 (included)\n")
    assign_here("loop")
    loadimm(TRASH, -1)
    sub(7, 7, TRASH)
    out_number(7)
    do_print(b" ")
    if_then_else(BUSY_REGS + [7], ("neconst", 7, 10), lambda _: jump("loop"))
    do_print(b"\n")
    exit()
    add_print_function()


def fact_example():
    do_print(b"I will compute some factorials for you\n")
    assign_here("loop")
    loadimm(TRASH, -1)
    sub(7, 7, TRASH)
    do_print(b"fact(")
    out_number(7)
    do_print(b") = ")
    move(10, 7)
    jsr("fact")
    out_number(11)
    do_print(b"\n")
    if_then_else(BUSY_REGS + [7], ("neconst", 7, 10), lambda _: jump("loop"))
    do_print(b"I'm done!\n")
    exit()
    add_fact_function()
    add_print_function()


def fibo_example():
    do_print(b"I will compute some Fibonacci numbers for you\n")
    assign_here("loop")
    loadimm(TRASH, -1)
    sub(7, 7, TRASH)
    do_print(b"fibo(")
    out_number(7)
    do_print(b") = ")
    move(10, 7)
    jsr("fibo")
    out_number(11)
    do_print(b"\n")
    if_then_else(BUSY_REGS + [7], ("neconst", 7, 23), lambda _: jump("loop"))
    do_print(b"I'm done!\n")
    exit()
    add_fibo_function()
    add_print_function()


def beer_example():
    loadimm(7, 99)
    assign_here("loop")
    jsr("ubottles")
    do_print(b" of beer on the wall, ")
    jsr("bottles")
    do_print(b" of beer.\n")
    if_then_else(BUSY_REGS + [7], ("eqconst", 7, 0),
                 lambda _: jump("no_more_bottles"))
    do_print(b"Take one down, pass it around, ")
    loadimm(TRASH, 1)
    sub(7, 7, TRASH)
    jsr("ubottles")
    do_print(b" of beer on the wall...\n\n")
    jump("loop")
    assign_here("no_more_bottles")
    do_print(b"Go to the store and buy some more, 99 bottles of beer on the wall...\n")
    exit()
    start_function("ubottles")

    def one(_):
        do_print(b"One bottle")

    def zero(_):
        do_print(b"No more bottles")

    def more(_):
        out_number(7)
        do_print(b" bottles")

    def zero_or_more(_):
        if_then_else(CALLEE_SAVE, 7, more, zero)
    if_then_else(CALLEE_SAVE, ("neconst", 7, 1), zero_or_more, one)
    end_function()
    start_function("bottles")

    def one(_):
        do_print(b"one bottle")

    def zero(_):
        do_print(b"no more bottles")

    def more(_):
        out_number(7)
        do_print(b" bottles")

    def zero_or_more(_):
        if_then_else(CALLEE_SAVE, 7, more, zero)
    if_then_else(CALLEE_SAVE, ("neconst", 7, 1), zero_or_more, one)
    end_function()
    add_print_function()


def make_example(f, basename):
    counters.clear()
    symbols.clear()
    code.clear()
    data.clear()
    loadimm(SP, MEMORY_SIZE)
    f()
    import sys
    with open("{}.dis".format(basename), "wt") as outfd:
        disassemble(outfd)
    append_data()
    replace_labels()
    open("{}.bin".format(basename), "wb").write(bytes(code))


make_example(push_pop_test, "tests/push_pop")
make_example(jump_to_function_test, "tests/function")
make_example(mult_test, "tests/multiply")
make_example(fact_test, "tests/fact")
make_example(afact_test, "tests/afact")
make_example(rfact_test, "tests/rfact")
make_example(rfact_tr_test, "tests/rfact_tr")
make_example(fibo_test, "tests/fibo")

make_example(hello_world_example, "examples/hello_world")
make_example(count_example, "examples/count")
make_example(fact_example, "examples/factorial")
make_example(fibo_example, "examples/fibonacci")
make_example(beer_example, "examples/99bottles")
