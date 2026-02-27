# Naze Examples

Curated code examples by category. Each example is a complete, compilable `.naze` file.

## Counter (State + Events)

```naze
app "Counter" {
  state count = 0
  let title = "My Counter"

  column padding: 20px, gap: 16px {
    heading "{title}"
    text "Current count: {count}"
    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Increment"
      on click: set count = count + 1
    }
    rect width: 200px, height: 50px, color: #dc2626, radius: 8px {
      text "Reset"
      on click: set count = 0
    }
  }
}
```

## Form with Validation

```naze
app "Sign Up" {
  state username = ""
  state email = ""
  state age = ""

  column padding: 20px, gap: 16px {
    heading "Create Account"

    text "Username (3-20 characters)"
    input bind: username, placeholder: "Enter username", validate: { required: true, min-length: 3, max-length: 20 }
    if username_error {
      text "{username_error}" color: #dc2626
    }

    text "Email"
    input bind: email, type: "email", placeholder: "Enter email", validate: { required: true }
    if email_error {
      text "{email_error}" color: #dc2626
    }

    text "Age (18-120)"
    input bind: age, type: "number", placeholder: "Enter age", validate: { required: true, min: 18, max: 120 }
    if age_error {
      text "{age_error}" color: #dc2626
    }

    row gap: 8px {
      if username_valid {
        text "Username valid" color: #16a34a
      }
      if email_valid {
        text "Email valid" color: #16a34a
      }
      if age_valid {
        text "Age valid" color: #16a34a
      }
    }
  }
}
```

## Data Fetching (API)

```naze
app "Data Fetch Demo" {
  data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

  column gap: 16px, padding: 20px {
    heading "API Data Example"

    if posts.loading {
      text "Loading posts..." color: #666666
    }

    if posts.error {
      text "Error: {posts.error}" color: #dc2626
    }

    if posts.data {
      column gap: 12px {
        each post in posts.data {
          column padding: 12px, color: #f3f4f6, radius: 8px {
            heading "{post.title}" font-size: 16px
            text "{post.body}" color: #666666, font-size: 14px
          }
        }
      }
    }
  }
}
```

## Multi-Page App (Routing)

```naze
app "Navigation Demo" {
  column color: #f8fafc {
    row padding: 16px, gap: 24px, color: #1e293b {
      heading "My App" color: #ffffff
      link "Home", to: "/"
      link "About", to: "/about"
      link "Contact", to: "/contact"
    }

    page "/" {
      column padding: 24px, gap: 16px {
        heading "Welcome Home"
        text "This is the home page."
        row gap: 8px {
          rect width: 100px, height: 100px, color: #3b82f6, radius: 8px
          rect width: 100px, height: 100px, color: #8b5cf6, radius: 8px
          rect width: 100px, height: 100px, color: #ec4899, radius: 8px
        }
      }
    }

    page "/about" {
      column padding: 24px, gap: 16px {
        heading "About Us"
        text "Naze is a declarative UI language that compiles to WebAssembly."
      }
    }

    page "/contact" {
      column padding: 24px, gap: 16px {
        heading "Contact"
        text "Get in touch!"
      }
    }
  }
}
```

## Component Definition and Usage

Component file (`components/color-box.naze`):
```naze
component color-box(color: color, size: number = 80px) {
  rect width: size, height: size, color: color, radius: 4px
}
```

App using the component:
```naze
use components/color-box

app "Component Basic" {
  column padding: 20px, gap: 16px {
    heading "Component Reuse"
    row gap: 12px {
      color-box color: #ef4444
      color-box color: #22c55e
      color-box color: #3b82f6
    }
  }
}
```

## Todo App (State, Lists, Themes, Animation)

```naze
app "Todo App" {
  state tasks = [
    {text: "Learn Naze", done: false},
    {text: "Build an app", done: false}
  ]
  state new-task = ""
  computed total-count = tasks | count

  column gap: 0px {
    container color: #1e293b, padding: 24px {
      heading "Todo App" color: #ffffff, font-size: 28px
      text "{total-count} items" color: #94a3b8
    }

    column padding: 24px, gap: 16px {
      row gap: 8px {
        input bind: new-task, placeholder: "What needs to be done?", validate: {required: true, min-length: 2}
        rect width: 80px, height: 40px, color: #2563eb, radius: 8px, role: "button", label: "Add task" {
          on click: append {text: new-task, done: false} to tasks
          on click: set new-task = ""
          column align: center, justify: center {
            text "Add" color: #ffffff
          }
        }
      }
      if new-task_error {
        text "{new-task_error}" color: #dc2626, font-size: 12px
      }

      column gap: 4px {
        each task in tasks {
          row gap: 12px, padding: 12px, color: #f8fafc, radius: 8px {
            text "{task.text}" font-size: 16px
            spacer
            rect width: 32px, height: 32px, color: #fee2e2, radius: 6px, role: "button", label: "Delete task" {
              on click: remove task_index from tasks
              column align: center, justify: center {
                text "x" color: #dc2626, font-size: 14px
              }
            }
          }
        }
      }

      if total-count == 0 {
        column padding: 32px, align: center {
          text "No tasks yet!" color: #94a3b8, font-size: 18px
        }
      }
    }
  }
}
```

## Server Functions with Database Models

```naze
model users {
  id number primary
  name text
  email text
}

server function list-users() {
  find users order id desc
}

server function add-user(name: text, email: text) {
  insert users {name: name, email: email}
}

server function remove-user(user-id: number) {
  delete users where id == user-id
}

app "Users" {
  state name = ""
  state email = ""
  data users: list-users()

  column padding: 20px, gap: 16px {
    heading "User Management"

    column gap: 8px {
      input bind: name, placeholder: "Name"
      input bind: email, type: "email", placeholder: "Email"
      rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
        text "Add User" color: #ffffff
        on click: trigger users
      }
    }

    if users.loading {
      text "Loading..." color: #64748b
    }

    if users.data {
      each user in users.data {
        row padding: 8px, color: #f3f4f6, radius: 4px, gap: 8px {
          text "{user.name}"
          text "{user.email}" color: #64748b
        }
      }
    }
  }
}
```

## Theming with Dark Mode

```naze
theme light {
  colors {
    bg: #ffffff
    text: #1e293b
    primary: #2563eb
    surface: #f8fafc
  }
}

theme dark extends light {
  colors {
    bg: #0f172a
    text: #f1f5f9
    primary: #60a5fa
    surface: #1e293b
  }
}

app "Themed App" {
  column padding: 20px, gap: 16px, color: theme.colors.bg {
    row gap: 8px {
      heading "Themed App" color: theme.colors.text
      spacer
      rect height: 36px, padding: 12px, color: theme.colors.surface, radius: 8px, role: "button", label: "Toggle theme" {
        on click: set-theme "dark"
        text "Dark Mode" color: theme.colors.text, font-size: 14px
      }
    }

    container padding: 16px, color: theme.colors.surface, radius: 8px {
      text "This card uses theme tokens for styling." color: theme.colors.text
    }

    rect width: 200px, height: 48px, color: theme.colors.primary, radius: 8px, role: "button", label: "Primary action" {
      column align: center, justify: center {
        text "Primary Button" color: #ffffff
      }
    }
  }
}
```

## Dashboard Layout

```naze
app "Dashboard" {
  data stats: fetch "/api/stats"

  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" font-size: 20px, color: #fff
    }

    column padding: 20px, gap: 16px {
      if stats.loading {
        text "Loading..."
      }
      if stats.data {
        row gap: 16px {
          container padding: 16px, color: #eff6ff, radius: 8px, width: 180px {
            text "Revenue"
            heading "$12,345" font-size: 24px
          }
          container padding: 16px, color: #f0fdf4, radius: 8px, width: 180px {
            text "Users"
            heading "1,234" font-size: 24px
          }
        }
      }
    }
  }
}
```

## Pipelines (Filter, Sort, Count)

```naze
app "Pipeline Demo" {
  state items = [
    {name: "Alice", score: 95},
    {name: "Bob", score: 72},
    {name: "Carol", score: 88},
    {name: "Dave", score: 64}
  ]

  computed top-count = items | filter score > 80 | count
  computed avg-score = items | map score | sum

  column padding: 20px, gap: 16px {
    heading "Scores"
    text "Above 80: {top-count}"
    text "Total points: {avg-score}"

    heading "Top Scorers (sorted)" font-size: 18px
    each student in items | filter score > 80 | sort-by name {
      row gap: 8px {
        text "{student.name}"
        text "{student.score}" color: #16a34a
      }
    }

    heading "All Students" font-size: 18px
    each student in items | sort-by score {
      text "{student.name}: {student.score}"
    }
  }
}
```
