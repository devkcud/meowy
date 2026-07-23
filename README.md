# meowy

**meowy** is a small and composable compiled programming language built around a handful of ideas.

Its guiding principle is: **"what you write is what you get."**

Instead of introducing a new language construct for every feature, meowy reuses the same primitives to express modules, functions, objects, configuration, control flow, and more.

The language intentionally has a very small core. Once you understand that core, the rest of the language becomes combinations of the same few concepts.

> Also, we have a very small set of well-known values and literals: `null`, `true`, `false`, and numeric values. Even contextual names such as `self` can be redefined.

Inspired by: Smalltalk, OCaml, Golang, and Lisp.

---

## The core idea

Everything in meowy revolves around **blocks** and **values**.

Every block may produce:

- **zero or one primary value**, which becomes the value of the block itself
- **zero or more named values**, which become accessible through `.`.

For example:

```
user : {
    -> "Dev"
    -> id : "abc123"
}

user    # "Dev" #
user.id # "abc123" #
```

Here, the block's primary value is `"Dev"` and its named value is `id`.

This single rule is surprisingly powerful.

Modules, functions, objects, configuration values, and many other language features all build upon it.

The absence of a primary value makes the block type `<null>`. For example:

```
user : {
    -> id : "abc123"
}

user    # null #
user.id # "abc123" #
```

---

## The big five

Everything else in meowy can be explained using five concepts.

|     Concept | Purpose                                                                                               |
| ----------: | :---------------------------------------------------------------------------------------------------- |
|      Values | Store data. Numbers, string, types, lists, functions, modules, nulls, and even errors are all values. |
|      Blocks | Evaluate code and produce values.                                                                     |
|    Emitters | Produce one primary value and any number of named values from a block.                                |
| Dispatchers | Evaluate something in the context of another value, which becomes available as `self`.                |
|    Matchers | Execute code only when a comparison or type check succeeds.                                           |

The rest of this guide introduces each concept individually before combining them into larger programs.

---

## A first example

Let's look at a realistic example before explaining every detail.

```meowy
dbg     : @"debug".print
convert : @"strings".convert.toint8
ask     : @"cli".ask

age := ask("What is your age? ")
age = age
    .(convert)
    .{
        | self <error> | dbg.panic("{age} is not a valid number!")
        -> self
    }
```

This may look like a lot. It is actually the same few concepts repeated several times.

By the end of this guide, every part of this example should feel familiar.

---

## Values

Bindings are immutable by default.

```meowy
name : "meowy"
```

Once created, `name` cannot be reassigned.

Mutable bindings use `:=` instead.

```meowy
age := ask("What is your age? ")
```

Then may later be updated.

```meowy
age = age.(convert)
```

The compiler infers the type of every binding unless you choose to annotate it.

Explicit types are used to express intent, not because the compiler requires them.

---

## Blocks

Blocks are the fundamental building block of the language.

A block evaluates code and emits values.

```meowy
user : {
    -> "Dev"
    -> id : "abc123"
}
```

The first emit is the block's **primary value**.

```meowy
-> "Dev"
```

The second emit creates a **named value**.

```meowy
-> id : "abc123"
```

Which can later be accessed.

```meowy
user    # "Dev" #
user.id # "abc123" #
```

Blocks are used everywhere in meowy. A module is a block. A function is a block. An object-like value is a block. Configuration values are often blocks.

The language intentionally avoids introducing separate constructs for each of these ideas.

---

## Emitters

**Blocks do not return values**. They **emit** them. This distinction is important.

In many programming languages, `return` immediately stops execution.

```ts
function greet() {
  return "Hello";

  console.log("Never runs");
}
```

In meowy, emitting a value **does not** stop the current block's execution.

```meowy
greeting : {
    -> "Hello"

    print("Still running!")

    -> author : "meowy"
}

greeting        # "Hello" #
greeting.author # "meowy" #
```

The block above emits:

- a primary value: `"Hello"`
- a named value: `author`.

Even though the primary value was emitted first, the block continues executing.

### Primary and named values

Every block may emit:

- one primary value
- zero or more named values.

```meowy
print(
    {
        -> "My Text!"
    }
)
```

The primary value becomes the value of the block itself.

Named values become accessible only through `.`.

```meowy
config : {
    -> language : "en"
    -> user : "Dev"
}

config.language # "en" #
config.user     # "Dev" #
```

### Primary values propagate

Primary values naturally propagate through nested blocks.

```meowy
connection : {
    cfg : read_config()

    -> {
        -> token : authenticate(cfg)

        -> {
            conn : connect(token)

            -> conn
        }
    }
}

connection.token # valid! #
connection # valid! conn here acts as the primary value #
```

Notice how the outer block never explicitly forwards the connection. Evaluating its primary value follows the chain of primary emissions until it reaches `conn`. Named values encountered along that path, such as `token`, remain accessible through dispatch.

Buckle with us because it can get quite... expensive in other languages:

```ts
type BlockValue<T> = {
  primary: T;
  named: Record<string, unknown>;
};

function connection() {
  const cfg = read_config();

  const inner = (() => {
    const token = authenticate(cfg);

    const deeper = (() => {
      const conn = connect(token);

      return {
        primary: conn,
        named: {},
      };
    })();

    return {
      primary: deeper,
      named: {
        token,
      },
    };
  })();

  return {
    primary: inner,
    named: {},
  };
}

function primary(value: any): any {
  while (value && typeof value === "object" && "primary" in value) {
    value = value.primary;
  }

  return value;
}

function named(value: any, key: string): any {
  while (value && typeof value === "object") {
    if (value.named && key in value.named) {
      return value.named[key];
    }

    value = value.primary;
  }
}

const result = connection();

primary(result); // conn
named(result, "token"); // token
```

> _"Look what they need to mimic a fraction of our power."_

### Why "emit" instead of "return"?

Because blocks are more than functions.

A block may simultaneously produce:

- a primary value,
- additional named values,
- and continue executing after doing so.

Modules, objects, maps, functions, configurations, etc. all rely on this behavior.

Thinking in terms of **emitting values** instead of **returning values** makes the rest of the language much easier to understand.

---

## Dispatchers

Dispatch sends a value into another computation.

```meowy
token.(is_valid)
```

is equivalent to

```meowy
is_valid(token)
```

Both forms are valid.

A dispatch becomes especially useful when several transformations happen in sequence.

```meowy
user
    .(connect)
    .(authenticate)
    .(verify)
```

instead of

```meowy
verify(authenticate(connect(user)))
```

The computation grows left-to-right instead of inside-out.

Dispatches may also target an inline block.

```meowy
value.{
    | self <error> | panic()
    -> self
}
```

Inside the dispatched block, the incoming value is available as `self`.

Technically, it is more of: "evaluate the right-hand side with `self` bound to the left-hand value." So:

```meowy
user.name # is a dispatch #
user.{ -> self.name } # is a dispatch #
user.(get_name) # is a dispatch #
# etc... #
```

---

## Matchers

Matchers execute their blocks only when their condition succeeds.

Unlike `if`, matchers do not introduce a separate control-flow construct.

They are simply **independent conditional arms** that may appear wherever a block may appear.

For example:

### Expression matching

```meowy
| age >= 18 | print("Adult")
```

The block executes only if the expression evaluates to `true`.

### Type matching

```meowy
| self <error> | panic(self)
```

The block executes only if `self` is an instance of the `<error>` type.

### Mix matching

```meowy
| self <error> | panic("Halt!")
| self <int8> && self >= 1 | execute_a()
| self <int8> && self >= 2 | execute_b()
| self <string> && self == "hi" | {
    print("hi..?")
    print("you need something?")
}
```

Here we are mixing both type and expression matching to execute different blocks.

### Combining matchers

```meowy
theme : "dark"

colors : {
    | theme == "dark" | -> {
        -> bg : BLACK
        -> fg : WHITE
    }

    | theme == "light" | -> {
        -> bg : WHITE
        -> fg : BLACK
    }
}
```

Here, the matchers collectively provide the kind of conditional selection that might otherwise be written using `if`, `switch`, or `match`.

Note that they are independently tested against their conditions. So read the example above as:

```ts
const theme = "dark";

function colors() {
  if (theme === "dark") {
    return {
      bg: BLACK,
      fg: WHITE,
    };
  }

  if (theme === "light") {
    return {
      bg: WHITE,
      fg: BLACK,
    };
  }
}
```

Conceptually, this is similar to the meowy version. However, meowy doesn't need a separate if, return, function, or object-literal construct here. Matchers decide which blocks execute, while emitters determine what those blocks expose.

---

## Putting it together

Returning to the original example:

```meowy
age := ask("What is your age? ")

age = age
    .(convert)
    .{
        | self <error> | dbg.panic("{age} is not a valid number!")
        -> self
    }
```

The flow is:

1. Ask the user for text.
2. Dispatch that text into `convert`.
3. Dispatch the result into another block.
4. If the value is an error, panic.
5. Otherwise emit the successfully converted integer.
6. Assign the final value back to `age`.

Nothing new was introduced during this process.

The program is built entirely from values, blocks, emitters, dispatchers, and matchers.

The same can be written as:

```meowy
age := ask("What is your age? ")

converted : convert(age)

| converted <error> | dbg.panic("{age} is not a valid number!")

age = converted
```

---

## Other concepts

In later chapters we will dive into:

- named blocks (`'scope { ... }`), and their `restart()` and `leave()` functions,
- modules and file importing with `@""` notation,
- channeling tasks (async work),
- and `mod.mwy` setup and the compiler's well-known emitted values.

Of course, there is much more to the language. We haven't even dipped our toes into the typing system yet. Or lists. Or even wait groups.

## Where this goes

It wouldn't be very meowy of us to explain the language without showing you what it can actually look like.

You don't need to understand everything here yet. Some of it belongs to later chapters. But the core ideas you've just learned are already doing most of the work.

```meowy
pg : @"codeberg.org/meowy-lang-extra/database".pg;
print : @"codeberg.org/meowy-lang/debug".print;

env : @"github.com/meowy-lang-legacy/env".lazy_load();

conn : env.get("POSTGRES_URI").(pg.connect).{
    | self <error> | print.panic(self);
    -> self;
};

| << conn.ping() | {
    print("pong! connecting working!");
};

user : {
    u : conn.query("select id, name from users where name = %name% limit 1", ["name" : "dev"]).row(1);
    -> u["name"];

    <Project> : <{
        id <uint128>;
        name <string>;
        status <"active"><"paused"><"completed">;
    }>;

    p : conn.query("
        select
            p.id,
            p.name,
            p.status
        from projects p
        where p.user_id = %id%
        order by p.name
    ", ["id" : u["id"]]);

    -> projects : {
        rows <Project> := [];

        'loop {
            rows = rows.add(p.row(rows.size() + 1).{
                -> id : self["p.id"];
                -> name : self["p.name"];
                -> status : self["p.status"];
            });

            | rows.size() != p.rows.size() | 'loop.restart();
        };

        -> rows;
    };
};
```

If parts of this still look unfamiliar, good. We haven't covered types, named blocks, channeling, queries, or lists in detail yet.

But blocks, emissions, dispatches, matchers, bindings, and self should already look familiar.

The rest is more of the same ideas, composed.

[Keep reading](./docs/guide/README.md)
