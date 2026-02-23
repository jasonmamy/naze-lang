#!/usr/bin/env python3
"""Generate ~800 validated .naze example files for training data expansion."""

import argparse
import subprocess
import sys
import time
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
OUTPUT_DIR = PROJECT_ROOT / "examples" / "generated"
NAZEC = PROJECT_ROOT / "target" / "debug" / "nazec"

EXAMPLES = []


def ex(name, desc, code):
    EXAMPLES.append((name, desc, code.strip() + "\n"))


def fill(template, reps):
    """Replace __KEY__ placeholders in template."""
    code = template
    for k, v in reps.items():
        code = code.replace(f"__{k}__", str(v))
    return code


# ═══════════════════════════════════════════════════════════════════════════════
# HAND-CRAFTED: Test batch (10) + Missing features (25) = 35 examples
# ═══════════════════════════════════════════════════════════════════════════════

ex("gen-stack-layers.naze",
   "A layered card using stack layout with overlapping text on a background",
   """-- Layered card with stack layout
app "Layered Card" {
  stack {
    rect width: 300px, height: 200px, color: #3b82f6, radius: 12px
    column padding: 16px {
      heading "Featured" color: #ffffff, font-size: 24px
      text "Overlaid on colored background" color: #ffffff
    }
  }
}""")

ex("gen-code-snippet.naze",
   "A code snippet viewer with monospace text display",
   """-- Code snippet display
app "Code Viewer" {
  column padding: 20px, gap: 16px {
    heading "Code Example"
    rect padding: 16px, color: #1e293b, radius: 8px {
      code "def hello():" color: #e2e8f0, font-size: 14px
      code "    print('Hello')" color: #e2e8f0, font-size: 14px
    }
    text "A simple Python snippet" color: #64748b
  }
}""")

ex("gen-meta-page.naze",
   "Multi-page app with SEO meta tags on each page",
   """-- SEO metadata on pages
app "Blog" {
  meta title: "My Blog"
  meta description: "A blog about coding"

  column padding: 20px {
    heading "Blog Home"
    text "Welcome to the blog"
    link "About us", to: "/about"
  }
}

page "/about" {
  meta title: "About Us"
  meta description: "Learn about our team"

  column padding: 20px {
    heading "About"
    text "About page content"
  }
}""")

ex("gen-param-data.naze",
   "User profile page with URL parameter used in a data fetch",
   """-- URL parameter with data fetching
app "User Profile" {
  param user-id: number default: 1
  data profile: fetch "/api/users/{user-id}"

  column padding: 20px, gap: 16px {
    heading "User Profile"

    if profile.loading {
      text "Loading..." color: #64748b
    }

    if profile.data {
      text "Profile for user #{user-id}" font-size: 20px
    }
  }
}""")

ex("gen-counter-minimal.naze",
   "A minimal counter with a single increment button",
   """-- Minimal counter
app "Counter" {
  state count = 0

  column padding: 20px {
    text "{count}" font-size: 32px
    rect width: 80px, height: 36px, color: #3b82f6, radius: 4px {
      text "+1" color: #ffffff
      on click: set count = count + 1
    }
  }
}""")

ex("gen-counter-complex.naze",
   "Advanced counter with auto-increment timer, computed state, and speed controls",
   """-- Counter with timer and computed state
app "Advanced Counter" {
  state count = 0
  state speed = 1
  computed doubled = count * 2

  timer auto-tick: every 1s {
    set count = count + speed
  }

  column padding: 24px, gap: 16px {
    heading "Advanced Counter" color: #6366f1
    text "Count: {count}" font-size: 24px
    text "Doubled: {doubled}" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 40px, color: #6366f1, radius: 8px {
        text "+1" color: #ffffff
        on click: set count = count + 1
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset" color: #ffffff
        on click: set count = 0
      }
    }

    row gap: 8px {
      text "Speed:" color: #64748b
      rect width: 60px, height: 32px, color: #e2e8f0, radius: 4px {
        text "1x"
        on click: set speed = 1
      }
      rect width: 60px, height: 32px, color: #e2e8f0, radius: 4px {
        text "5x"
        on click: set speed = 5
      }
    }
  }
}""")

ex("gen-form-search-list.naze",
   "Searchable list with pipeline sorting and item count",
   """-- Search form with sorted list
app "Fruit Search" {
  state query = ""
  state items = [{name: "Apple", kind: "fruit"}, {name: "Banana", kind: "fruit"}, {name: "Carrot", kind: "vegetable"}]

  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "Search & Filter"
    input bind: query, placeholder: "Type to search..."
    text "{total} items" color: #64748b

    each item in items | sort-by name {
      row padding: 8px, color: #f3f4f6, radius: 4px, gap: 8px {
        text "{item.name}" font-weight: bold
        text "{item.kind}" color: #64748b
      }
    }
  }
}""")

ex("gen-theme-storage.naze",
   "Theme switcher with persistent storage and light/dark modes",
   """-- Theme switching with storage
theme light {
  colors {
    bg: #ffffff
    fg: #0f172a
    primary: #2563eb
    card: #f1f5f9
  }
}

theme dark extends light {
  colors {
    bg: #0f172a
    fg: #f8fafc
    primary: #60a5fa
    card: #1e293b
  }
}

app "Theme Demo" {
  storage theme-pref: local "app-theme" default: "light"

  column padding: 20px, gap: 16px, color: theme.colors.bg {
    heading "Theme Settings" color: theme.colors.fg

    row gap: 8px {
      rect width: 100px, height: 40px, color: #f1f5f9, radius: 8px {
        text "Light"
        on click: set-theme "light"
      }
      rect width: 100px, height: 40px, color: #1e293b, radius: 8px {
        text "Dark" color: #ffffff
        on click: set-theme "dark"
      }
    }

    rect padding: 16px, color: theme.colors.card, radius: 8px {
      text "This card uses theme colors" color: theme.colors.fg
    }
  }
}""")

ex("gen-timer-conditional.naze",
   "Countdown timer with match-based display and start/reset controls",
   """-- Countdown with conditional display
app "Countdown" {
  state seconds = 10
  state mode = "ready"

  timer tick: every 1s {
    set seconds = seconds - 1
  }

  column padding: 20px, gap: 16px {
    heading "Countdown Timer"

    match mode {
      "ready": text "Press Start" color: #64748b, font-size: 20px
      "running": text "{seconds}s" font-size: 48px, color: #2563eb
      "done": text "Complete!" color: #16a34a, font-size: 24px
      _: text "Unknown"
    }

    row gap: 8px {
      rect width: 100px, height: 40px, color: #16a34a, radius: 8px {
        text "Start" color: #ffffff
        on click: set mode = "running"
      }
      rect width: 100px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset" color: #ffffff
        on click: set seconds = 10
      }
    }
  }
}""")

ex("gen-dashboard-metrics.naze",
   "Metrics dashboard with grid layout showing visitors, revenue, orders, and averages",
   """-- Dashboard with metric cards
app "Dashboard" {
  state visitors = 1250
  state revenue = 8400
  state orders = 64
  computed avg-order = revenue / orders

  column padding: 20px, gap: 16px {
    heading "Dashboard"

    grid columns: 2, gap: 16px {
      rect padding: 16px, color: #eff6ff, radius: 8px {
        text "Visitors" color: #64748b, font-size: 14px
        text "{visitors}" font-size: 28px, color: #1e40af
      }
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "Revenue" color: #64748b, font-size: 14px
        text "${revenue}" font-size: 28px, color: #166534
      }
      rect padding: 16px, color: #fef3c7, radius: 8px {
        text "Orders" color: #64748b, font-size: 14px
        text "{orders}" font-size: 28px, color: #92400e
      }
      rect padding: 16px, color: #fce7f3, radius: 8px {
        text "Avg Order" color: #64748b, font-size: 14px
        text "${avg-order}" font-size: 28px, color: #9d174d
      }
    }
  }
}""")

# ─── Missing features: prompt ────────────────────────────────────────────────

ex("gen-prompt-basic.naze",
   "AI text summarizer using prompt declaration with OpenAI provider",
   """-- AI summarizer with prompt
app "Summarizer" {
  state input-text = ""
  state result = ""

  prompt summary: from openai {
    system: "You are a helpful summarizer."
    user: "{input-text}"
    model: "gpt-4"
    max-tokens: 200
    temperature: 0.3
  }

  column padding: 20px, gap: 16px {
    heading "Text Summarizer"
    textarea bind: input-text, placeholder: "Paste text to summarize..."
    rect width: 120px, height: 40px, color: #8b5cf6, radius: 8px {
      text "Summarize" color: #ffffff
      on click: trigger summary
    }

    if result {
      text "{result}" color: #1e293b
    }
  }
}""")

ex("gen-prompt-chat.naze",
   "Chat assistant using prompt with Ollama provider and conversation state",
   """-- Chat with Ollama
app "Chat" {
  state message = ""
  state reply = ""

  prompt chat-bot: from ollama {
    system: "You are a helpful assistant."
    user: "{message}"
    model: "llama3"
    max-tokens: 500
    temperature: 0.7
  }

  column padding: 20px, gap: 16px {
    heading "Chat Assistant"
    input bind: message, placeholder: "Ask a question..."
    rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
      text "Send" color: #ffffff
      on click: trigger chat-bot
    }

    if reply {
      rect padding: 12px, color: #f0f9ff, radius: 8px {
        text "{reply}"
      }
    }
  }
}""")

# ─── Missing features: boundary ──────────────────────────────────────────────

ex("gen-boundary-fetch.naze",
   "Error boundary wrapping a data fetch with fallback UI",
   """-- Error boundary around data
app "Safe Fetch" {
  data posts: fetch "/api/posts"

  column padding: 20px, gap: 16px {
    heading "Posts"

    boundary {
      if posts.loading {
        text "Loading posts..." color: #64748b
      }

      if posts.data {
        each post in posts.data {
          text "{post.title}"
        }
      }
    } catch {
      text "Something went wrong" color: #dc2626
      text "Please try again later" color: #64748b
    }
  }
}""")

ex("gen-boundary-nested.naze",
   "Nested error boundaries with independent fallbacks",
   """-- Nested error boundaries
app "Resilient App" {
  data users: fetch "/api/users"
  data stats: fetch "/api/stats"

  column padding: 20px, gap: 16px {
    heading "Dashboard"

    boundary {
      text "Users: loading..." color: #64748b
    } catch {
      text "Users unavailable" color: #dc2626
    }

    boundary {
      text "Stats: loading..." color: #64748b
    } catch {
      text "Stats unavailable" color: #dc2626
    }
  }
}""")

# ─── Missing features: shared state ──────────────────────────────────────────

ex("gen-shared-state.naze",
   "Multi-page app with shared authentication state across pages",
   """-- Shared state across pages
shared state logged-in = false
shared state username = ""

app "Portal" {
  column padding: 20px, gap: 16px {
    heading "Home"

    if logged-in {
      text "Welcome, {username}!" font-size: 20px
      link "Profile", to: "/profile"
    }

    if logged-in == false {
      text "Please log in" color: #64748b
      link "Login", to: "/login"
    }
  }
}

page "/login" {
  column padding: 20px, gap: 16px {
    heading "Login"
    rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
      text "Log In" color: #ffffff
      on click: set logged-in = true
    }
  }
}

page "/profile" {
  column padding: 20px {
    heading "Profile"
    text "Logged in as {username}"
  }
}""")

# ─── Missing features: events ────────────────────────────────────────────────

ex("gen-context-menu.naze",
   "Context menu handler that logs right-click events",
   """-- Context menu event
app "Context Menu" {
  state last-action = "none"

  column padding: 20px, gap: 16px {
    heading "Right-Click Demo"

    rect width: 200px, height: 100px, color: #e2e8f0, radius: 8px {
      text "Right-click me"
      on context-menu: set last-action = "context-menu"
    }

    text "Last action: {last-action}" color: #64748b
  }
}""")

ex("gen-pointer-move.naze",
   "Pointer position tracker using pointer-move event",
   """-- Pointer move tracking
app "Pointer Tracker" {
  state pos-x = 0
  state pos-y = 0

  column padding: 20px, gap: 16px {
    heading "Pointer Position"

    rect width: 400px, height: 300px, color: #f1f5f9, radius: 8px {
      text "Move pointer here"
      on pointer-move: set pos-x = pos-x + 1
    }

    row gap: 16px {
      text "X: {pos-x}" color: #2563eb
      text "Y: {pos-y}" color: #2563eb
    }
  }
}""")

# ─── Missing features: data sources ──────────────────────────────────────────

ex("gen-data-js.naze",
   "JavaScript interop data source showing current timestamp",
   """-- JS data source
app "Clock" {
  data timestamp: js "Date.now"

  column padding: 20px, gap: 16px {
    heading "JS Clock"
    text "Timestamp: {timestamp.data}" font-size: 20px

    if timestamp.loading {
      text "Loading..." color: #64748b
    }
  }
}""")

ex("gen-data-device.naze",
   "Device geolocation data source showing coordinates",
   """-- Device geolocation
app "Location" {
  data coords: device "geolocation"

  column padding: 20px, gap: 16px {
    heading "My Location"

    if coords.loading {
      text "Getting location..." color: #64748b
    }

    if coords.data {
      text "Location acquired" color: #16a34a
    }

    if coords.error {
      text "Location unavailable" color: #dc2626
    }
  }
}""")

# ─── Missing features: server functions ───────────────────────────────────────

ex("gen-server-fetch.naze",
   "Server function that fetches data from an external API",
   """-- Server function with fetch
server function get-weather(city: text) {
  let result = fetch "https://api.weather.com/v1/{city}"
  result
}

app "Weather" {
  state city = "London"
  data weather: get-weather(city)

  column padding: 20px, gap: 16px {
    heading "Weather"
    input bind: city, placeholder: "City name"
    rect width: 100px, height: 36px, color: #0ea5e9, radius: 4px {
      text "Search" color: #ffffff
      on click: trigger weather
    }
  }
}""")

ex("gen-server-sql.naze",
   "Server function with raw SQL query",
   """-- Server function with SQL
server function recent-logs(n: number) {
  let result = sql "SELECT * FROM logs ORDER BY created_at DESC LIMIT $1" [n]
  result
}

app "Logs" {
  data logs: recent-logs(50)

  column padding: 20px, gap: 16px {
    heading "Recent Logs"

    if logs.loading {
      text "Loading..." color: #64748b
    }

    if logs.data {
      each log in logs.data {
        text "{log}" color: #374151
      }
    }
  }
}""")

ex("gen-server-update.naze",
   "Server function with update query setting fields",
   """-- Server function with update
model tasks {
  id number primary
  title text
  done bool default false
}

server function complete-task(task-id: number) {
  update tasks set {done: true} where id == task-id
}

server function list-tasks() {
  find tasks order id desc
}

app "Tasks" {
  data tasks: list-tasks()

  column padding: 20px, gap: 16px {
    heading "Task List"

    if tasks.data {
      each task in tasks.data {
        row padding: 8px, color: #f3f4f6, radius: 4px {
          text "{task.title}"
        }
      }
    }
  }
}""")

ex("gen-server-multi.naze",
   "Multi-step server function with let bindings",
   """-- Multi-step server function
server function enrich-user(user-id: number) {
  let user = fetch "https://api.example.com/users/{user-id}"
  let activity = fetch "https://api.example.com/activity/{user-id}"
  user
}

app "User Detail" {
  data detail: enrich-user(1)

  column padding: 20px, gap: 16px {
    heading "User Detail"

    if detail.loading {
      text "Loading user data..." color: #64748b
    }

    if detail.data {
      text "User loaded" color: #16a34a
    }
  }
}""")

ex("gen-server-limit.naze",
   "Server function with query limit clause",
   """-- Server function with limit
model articles {
  id number primary
  title text
  published bool default false
}

server function top-articles(n: number) {
  find articles where published == true order id desc limit n
}

app "Articles" {
  data articles: top-articles(10)

  column padding: 20px, gap: 16px {
    heading "Top Articles"

    if articles.data {
      each article in articles.data {
        text "{article.title}" font-size: 16px
      }
    }
  }
}""")

# ─── Missing features: notify, storage, template, guard ──────────────────────

ex("gen-notify-block.naze",
   "Notification with body and icon options",
   """-- Notification with options
app "Notifier" {
  state message = "Hello!"

  column padding: 20px, gap: 16px {
    heading "Notifications"
    input bind: message, placeholder: "Message..."

    rect width: 120px, height: 40px, color: #f59e0b, radius: 8px {
      text "Notify" color: #ffffff
      on click: notify "Alert" {
        body: "{message}"
        icon: "/icon.png"
      }
    }
  }
}""")

ex("gen-session-storage.naze",
   "Session storage for temporary preferences",
   """-- Session storage
app "Preferences" {
  storage view-mode: session "view-mode" default: "list"

  column padding: 20px, gap: 16px {
    heading "View Settings"
    text "Current mode: {view-mode}" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "List" color: #ffffff
        on click: set view-mode = "list"
      }
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "Grid" color: #ffffff
        on click: set view-mode = "grid"
      }
    }
  }
}""")

ex("gen-theme-spacing.naze",
   "Theme definition with spacing tokens",
   """-- Theme with spacing
theme {
  colors {
    primary: #6366f1
    bg: #fafafa
  }
  spacing {
    xs: 4px
    sm: 8px
    md: 16px
    lg: 24px
    xl: 48px
  }
}

app "Spaced" {
  column padding: 24px, gap: 16px {
    heading "Spacing Demo"
    text "This uses theme spacing tokens"
    spacer height: 16px
    text "After spacer" color: #64748b
  }
}""")

ex("gen-template-slots.naze",
   "Template component with multiple named slots",
   """-- Template with named slots
template card-layout(header, actions) {
  rect padding: 16px, color: #ffffff, radius: 8px, shadow: 2px {
    column gap: 12px {
      slot "header"
      separator
      text "Card body content"
      slot "actions"
    }
  }
}

app "Cards" {
  column padding: 20px, gap: 16px {
    heading "Card Demo"

    card-layout {
      fill "header" {
        heading "My Card" font-size: 18px
      }
      fill "actions" {
        row gap: 8px {
          rect width: 80px, height: 32px, color: #2563eb, radius: 4px {
            text "Save" color: #ffffff
          }
          rect width: 80px, height: 32px, color: #e2e8f0, radius: 4px {
            text "Cancel"
          }
        }
      }
    }
  }
}""")

ex("gen-guard-check.naze",
   "Route guard that redirects unauthenticated users",
   """-- Guard with authentication check
shared state authenticated = false

guard require-auth
  check authenticated redirect "/login"

app "Home" {
  column padding: 20px {
    heading "Public Home"
    link "Dashboard", to: "/dashboard"
    link "Login", to: "/login"
  }
}

page "/login" {
  column padding: 20px, gap: 16px {
    heading "Login"
    rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
      text "Log In" color: #ffffff
      on click: set authenticated = true
    }
  }
}

page "/dashboard" guard: require-auth {
  column padding: 20px {
    heading "Protected Dashboard"
    text "Only visible when authenticated"
  }
}""")

# ─── Missing features: layout elements ───────────────────────────────────────

ex("gen-spacer-separator.naze",
   "Using spacer and separator elements for visual spacing",
   """-- Spacer and separator
app "Layout Demo" {
  column padding: 20px, gap: 8px {
    heading "Section One"
    text "First section content"
    spacer height: 24px
    separator
    spacer height: 24px
    heading "Section Two"
    text "Second section content"
  }
}""")

ex("gen-scroll-list.naze",
   "Scrollable list of items in a fixed-height container",
   """-- Scroll container
app "Scrollable" {
  state items = ["Item 1", "Item 2", "Item 3", "Item 4", "Item 5", "Item 6", "Item 7", "Item 8"]

  column padding: 20px, gap: 16px {
    heading "Scroll Demo"
    scroll height: 200px {
      column gap: 4px {
        each item in items {
          rect padding: 12px, color: #f3f4f6, radius: 4px {
            text "{item}"
          }
        }
      }
    }
  }
}""")

ex("gen-container-card.naze",
   "Container element used as a card wrapper",
   """-- Container cards
app "Cards" {
  column padding: 20px, gap: 16px {
    heading "Card Layout"

    container padding: 16px, color: #ffffff, radius: 12px, shadow: 2px {
      column gap: 8px {
        heading "Card Title" font-size: 18px
        text "Card description goes here" color: #64748b
      }
    }

    container padding: 16px, color: #f0fdf4, radius: 12px {
      column gap: 8px {
        heading "Success Card" font-size: 18px, color: #166534
        text "Everything is working" color: #16a34a
      }
    }
  }
}""")

ex("gen-import-use.naze",
   "Import statement with use directive for external modules",
   """-- Import external module
import crypto from "./lib/crypto.wasm"

app "Secure App" {
  state token = ""

  column padding: 20px, gap: 16px {
    heading "Secure Operations"
    text "Token: {token}" color: #64748b

    rect width: 140px, height: 40px, color: #7c3aed, radius: 8px {
      text "Generate" color: #ffffff
      on click: set token = "generated"
    }
  }
}""")

ex("gen-each-pipeline.naze",
   "Each loop with chained pipeline operations",
   """-- Pipeline operations in each
app "Data Pipeline" {
  state scores = [{name: "Alice", score: 92}, {name: "Bob", score: 78}, {name: "Carol", score: 95}, {name: "Dan", score: 85}]

  computed top-count = scores | count
  computed best = scores | sort-by score | take 2

  column padding: 20px, gap: 16px {
    heading "Leaderboard"
    text "{top-count} participants" color: #64748b

    each entry in scores | sort-by score {
      row padding: 8px, color: #f3f4f6, radius: 4px, gap: 8px {
        text "{entry.name}" font-weight: bold
        text "{entry.score} pts" color: #2563eb
      }
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# GENERATORS: Parametric example generation (~365 examples)
# ═══════════════════════════════════════════════════════════════════════════════

# ─── Generator: Calculators (state + computed) ────────────────────────────────

CALC_T = """-- __DESC__
app "__TITLE__" {
  state __A__ = __VA__
  state __B__ = __VB__
  computed __C__ = __EXPR__

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__LA__: {__A__}" font-size: 18px
    text "__LB__: {__B__}" color: #64748b
    text "__LC__: {__C__}" font-size: 24px, color: __CLR__

    row gap: 8px {
      rect width: 80px, height: 36px, color: __CLR__, radius: 4px {
        text "+" color: #ffffff
        on click: set __A__ = __A__ + 1
      }
      rect width: 80px, height: 36px, color: #ef4444, radius: 4px {
        text "Reset" color: #ffffff
        on click: set __A__ = __VA__
      }
    }
  }
}"""

for n, cfg in [
    ("budget", {"TITLE": "Budget", "DESC": "Budget tracker with balance", "A": "income", "VA": "5000", "B": "expenses", "VB": "3200", "C": "balance", "EXPR": "income - expenses", "LA": "Income", "LB": "Expenses", "LC": "Balance", "CLR": "#2563eb"}),
    ("score", {"TITLE": "Quiz Score", "DESC": "Quiz score percentage calculator", "A": "correct", "VA": "0", "B": "total", "VB": "10", "C": "percent", "EXPR": "correct * 100 / total", "LA": "Correct", "LB": "Total", "LC": "Score", "CLR": "#16a34a"}),
    ("tip", {"TITLE": "Tip Calculator", "DESC": "Tip calculator with bill and rate", "A": "bill", "VA": "50", "B": "rate", "VB": "15", "C": "tip", "EXPR": "bill * rate / 100", "LA": "Bill", "LB": "Tip Rate", "LC": "Tip", "CLR": "#f59e0b"}),
    ("temp", {"TITLE": "Temperature", "DESC": "Celsius to Fahrenheit converter", "A": "celsius", "VA": "20", "B": "offset", "VB": "32", "C": "fahrenheit", "EXPR": "celsius * 9 / 5 + offset", "LA": "Celsius", "LB": "Offset", "LC": "Fahrenheit", "CLR": "#ef4444"}),
    ("bmi", {"TITLE": "BMI", "DESC": "BMI calculator from weight and height", "A": "weight", "VA": "70", "B": "height", "VB": "170", "C": "bmi", "EXPR": "weight * 10000 / (height * height)", "LA": "Weight (kg)", "LB": "Height (cm)", "LC": "BMI", "CLR": "#8b5cf6"}),
    ("savings", {"TITLE": "Savings", "DESC": "Savings calculator with monthly deposits", "A": "monthly", "VA": "500", "B": "months", "VB": "12", "C": "total", "EXPR": "monthly * months", "LA": "Monthly", "LB": "Months", "LC": "Total", "CLR": "#10b981"}),
    ("area", {"TITLE": "Area", "DESC": "Rectangle area calculator", "A": "width", "VA": "10", "B": "length", "VB": "20", "C": "area", "EXPR": "width * length", "LA": "Width", "LB": "Length", "LC": "Area", "CLR": "#0ea5e9"}),
    ("speed", {"TITLE": "Speed", "DESC": "Speed calculator from distance and time", "A": "distance", "VA": "100", "B": "time-taken", "VB": "2", "C": "speed", "EXPR": "distance / time-taken", "LA": "Distance", "LB": "Time", "LC": "Speed", "CLR": "#6366f1"}),
    ("discount", {"TITLE": "Discount", "DESC": "Price after discount calculator", "A": "price", "VA": "100", "B": "percent-off", "VB": "20", "C": "final", "EXPR": "price - (price * percent-off / 100)", "LA": "Price", "LB": "Discount %", "LC": "Final", "CLR": "#ec4899"}),
    ("fuel", {"TITLE": "Fuel Cost", "DESC": "Fuel cost estimator", "A": "distance-km", "VA": "200", "B": "per-liter", "VB": "8", "C": "cost", "EXPR": "distance-km * per-liter / 100", "LA": "Distance", "LB": "L/100km", "LC": "Cost", "CLR": "#f97316"}),
    ("grade", {"TITLE": "Grade", "DESC": "Weighted grade calculator", "A": "exam", "VA": "85", "B": "homework", "VB": "90", "C": "final-grade", "EXPR": "exam * 60 / 100 + homework * 40 / 100", "LA": "Exam", "LB": "Homework", "LC": "Final", "CLR": "#14b8a6"}),
    ("ratio", {"TITLE": "Ratio", "DESC": "Ratio calculator", "A": "part", "VA": "25", "B": "whole", "VB": "100", "C": "ratio", "EXPR": "part * 100 / whole", "LA": "Part", "LB": "Whole", "LC": "Ratio %", "CLR": "#a855f7"}),
    ("profit", {"TITLE": "Profit", "DESC": "Profit margin calculator", "A": "revenue", "VA": "10000", "B": "costs", "VB": "7000", "C": "profit", "EXPR": "revenue - costs", "LA": "Revenue", "LB": "Costs", "LC": "Profit", "CLR": "#22c55e"}),
    ("pace", {"TITLE": "Run Pace", "DESC": "Running pace calculator", "A": "minutes", "VA": "30", "B": "km", "VB": "5", "C": "pace", "EXPR": "minutes / km", "LA": "Minutes", "LB": "Kilometers", "LC": "Min/km", "CLR": "#06b6d4"}),
    ("split", {"TITLE": "Bill Split", "DESC": "Bill splitting calculator", "A": "total-bill", "VA": "120", "B": "people", "VB": "4", "C": "per-person", "EXPR": "total-bill / people", "LA": "Total", "LB": "People", "LC": "Each Pays", "CLR": "#d946ef"}),
]:
    ex(f"gen-calc-{n}.naze", cfg["DESC"], fill(CALC_T, cfg))

# ─── Generator: Lists (each + pipeline) ──────────────────────────────────────

LIST_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    each __IT__ in items | sort-by __SORT__ {
      row padding: 8px, color: __BG__, radius: 4px, gap: 8px {
        text "{__IT__.__F1__}" font-weight: bold
        text "{__IT__.__F2__}" color: #64748b
      }
    }
  }
}"""

for n, cfg in [
    ("users", {"TITLE": "Users", "DESC": "User directory sorted by name", "ITEMS": '{name: "Alice", role: "admin"}, {name: "Bob", role: "editor"}, {name: "Carol", role: "viewer"}', "LABEL": "users", "IT": "user", "SORT": "name", "F1": "name", "F2": "role", "BG": "#f3f4f6"}),
    ("products", {"TITLE": "Products", "DESC": "Product catalog sorted by name", "ITEMS": '{name: "Widget", price: "9.99"}, {name: "Gadget", price: "24.99"}, {name: "Tool", price: "14.99"}', "LABEL": "products", "IT": "product", "SORT": "name", "F1": "name", "F2": "price", "BG": "#f0fdf4"}),
    ("tasks", {"TITLE": "Tasks", "DESC": "Task list sorted by priority", "ITEMS": '{title: "Fix bug", priority: "high"}, {title: "Add test", priority: "medium"}, {title: "Update docs", priority: "low"}', "LABEL": "tasks", "IT": "task", "SORT": "priority", "F1": "title", "F2": "priority", "BG": "#eff6ff"}),
    ("books", {"TITLE": "Books", "DESC": "Book collection sorted by title", "ITEMS": '{title: "Dune", author: "Herbert"}, {title: "1984", author: "Orwell"}, {title: "Foundation", author: "Asimov"}', "LABEL": "books", "IT": "book", "SORT": "title", "F1": "title", "F2": "author", "BG": "#fef3c7"}),
    ("movies", {"TITLE": "Movies", "DESC": "Movie list sorted by title", "ITEMS": '{title: "Inception", year: "2010"}, {title: "Matrix", year: "1999"}, {title: "Arrival", year: "2016"}', "LABEL": "movies", "IT": "movie", "SORT": "title", "F1": "title", "F2": "year", "BG": "#fce7f3"}),
    ("songs", {"TITLE": "Playlist", "DESC": "Music playlist sorted by artist", "ITEMS": '{title: "Imagine", artist: "Lennon"}, {title: "Bohemian", artist: "Queen"}, {title: "Yesterday", artist: "Beatles"}', "LABEL": "songs", "IT": "song", "SORT": "artist", "F1": "title", "F2": "artist", "BG": "#f5f3ff"}),
    ("contacts", {"TITLE": "Contacts", "DESC": "Contact list sorted by name", "ITEMS": '{name: "John", phone: "555-0100"}, {name: "Jane", phone: "555-0101"}, {name: "Jim", phone: "555-0102"}', "LABEL": "contacts", "IT": "contact", "SORT": "name", "F1": "name", "F2": "phone", "BG": "#ecfdf5"}),
    ("recipes", {"TITLE": "Recipes", "DESC": "Recipe collection sorted by name", "ITEMS": '{name: "Pasta", time: "20min"}, {name: "Salad", time: "10min"}, {name: "Soup", time: "30min"}', "LABEL": "recipes", "IT": "recipe", "SORT": "name", "F1": "name", "F2": "time", "BG": "#fff7ed"}),
    ("cities", {"TITLE": "Cities", "DESC": "City directory sorted by name", "ITEMS": '{name: "Tokyo", country: "Japan"}, {name: "Paris", country: "France"}, {name: "London", country: "UK"}', "LABEL": "cities", "IT": "city", "SORT": "name", "F1": "name", "F2": "country", "BG": "#f0f9ff"}),
    ("tools", {"TITLE": "Dev Tools", "DESC": "Developer tools list sorted by name", "ITEMS": '{name: "VSCode", kind: "editor"}, {name: "Git", kind: "vcs"}, {name: "Docker", kind: "container"}', "LABEL": "tools", "IT": "tool", "SORT": "name", "F1": "name", "F2": "kind", "BG": "#f8fafc"}),
    ("languages", {"TITLE": "Languages", "DESC": "Programming languages sorted by name", "ITEMS": '{name: "Rust", paradigm: "systems"}, {name: "Python", paradigm: "scripting"}, {name: "Go", paradigm: "compiled"}', "LABEL": "languages", "IT": "lang", "SORT": "name", "F1": "name", "F2": "paradigm", "BG": "#faf5ff"}),
    ("orders", {"TITLE": "Orders", "DESC": "Order history sorted by status", "ITEMS": '{item: "Laptop", status: "shipped"}, {item: "Mouse", status: "delivered"}, {item: "Monitor", status: "pending"}', "LABEL": "orders", "IT": "order", "SORT": "status", "F1": "item", "F2": "status", "BG": "#fefce8"}),
    ("teams", {"TITLE": "Teams", "DESC": "Team roster sorted by department", "ITEMS": '{name: "Alpha", dept: "engineering"}, {name: "Beta", dept: "design"}, {name: "Gamma", dept: "marketing"}', "LABEL": "teams", "IT": "team", "SORT": "dept", "F1": "name", "F2": "dept", "BG": "#f1f5f9"}),
    ("events", {"TITLE": "Events", "DESC": "Event schedule sorted by date", "ITEMS": '{title: "Meetup", date: "Jan 15"}, {title: "Workshop", date: "Feb 20"}, {title: "Conference", date: "Mar 10"}', "LABEL": "events", "IT": "evt", "SORT": "date", "F1": "title", "F2": "date", "BG": "#fdf2f8"}),
    ("pets", {"TITLE": "Pets", "DESC": "Pet registry sorted by name", "ITEMS": '{name: "Max", species: "dog"}, {name: "Luna", species: "cat"}, {name: "Buddy", species: "dog"}', "LABEL": "pets", "IT": "pet", "SORT": "name", "F1": "name", "F2": "species", "BG": "#ecfeff"}),
]:
    ex(f"gen-list-{n}.naze", cfg["DESC"], fill(LIST_T, cfg))

# ─── Generator: Forms (input + bind) ─────────────────────────────────────────

FORM_T = """-- __DESC__
app "__TITLE__" {
  state __F1__ = ""
  state __F2__ = ""
  state submitted = false

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    input bind: __F1__, placeholder: "__P1__"
    input bind: __F2__, placeholder: "__P2__"__EXTRA__

    rect width: 120px, height: 40px, color: __CLR__, radius: 8px {
      text "__BTN__" color: #ffffff
      on click: set submitted = true
    }

    if submitted {
      text "__MSG__" color: #16a34a
    }
  }
}"""

for n, cfg in [
    ("login", {"TITLE": "Login", "DESC": "Login form with email and password", "F1": "email", "F2": "password", "P1": "Email", "P2": "Password", "BTN": "Log In", "CLR": "#2563eb", "MSG": "Logged in!", "EXTRA": ""}),
    ("register", {"TITLE": "Register", "DESC": "Registration form with name and email", "F1": "name", "F2": "email", "P1": "Full Name", "P2": "Email Address", "BTN": "Sign Up", "CLR": "#16a34a", "MSG": "Account created!", "EXTRA": ""}),
    ("contact", {"TITLE": "Contact Us", "DESC": "Contact form with name and message", "F1": "name", "F2": "message", "P1": "Your Name", "P2": "Message", "BTN": "Send", "CLR": "#8b5cf6", "MSG": "Message sent!", "EXTRA": ""}),
    ("feedback", {"TITLE": "Feedback", "DESC": "Feedback form with topic and comment", "F1": "topic", "F2": "comment", "P1": "Topic", "P2": "Your feedback", "BTN": "Submit", "CLR": "#f59e0b", "MSG": "Thanks for feedback!", "EXTRA": ""}),
    ("search", {"TITLE": "Search", "DESC": "Search form with query and category", "F1": "query", "F2": "category", "P1": "Search term", "P2": "Category", "BTN": "Search", "CLR": "#0ea5e9", "MSG": "Searching...", "EXTRA": ""}),
    ("settings", {"TITLE": "Settings", "DESC": "Settings form with username and bio", "F1": "username", "F2": "bio", "P1": "Username", "P2": "Bio", "BTN": "Save", "CLR": "#64748b", "MSG": "Settings saved!", "EXTRA": ""}),
    ("checkout", {"TITLE": "Checkout", "DESC": "Checkout form with address and card", "F1": "address", "F2": "card", "P1": "Shipping Address", "P2": "Card Number", "BTN": "Pay Now", "CLR": "#22c55e", "MSG": "Payment processed!", "EXTRA": ""}),
    ("booking", {"TITLE": "Booking", "DESC": "Booking form with date and guests", "F1": "date", "F2": "guests", "P1": "Date", "P2": "Number of guests", "BTN": "Book", "CLR": "#ec4899", "MSG": "Booking confirmed!", "EXTRA": ""}),
    ("profile", {"TITLE": "Edit Profile", "DESC": "Profile edit form with name and location", "F1": "name", "F2": "location", "P1": "Display Name", "P2": "Location", "BTN": "Update", "CLR": "#6366f1", "MSG": "Profile updated!", "EXTRA": ""}),
    ("newsletter", {"TITLE": "Newsletter", "DESC": "Newsletter signup with name and email", "F1": "name", "F2": "email", "P1": "Name", "P2": "Email", "BTN": "Subscribe", "CLR": "#14b8a6", "MSG": "Subscribed!", "EXTRA": ""}),
    ("survey", {"TITLE": "Survey", "DESC": "Survey form with rating and comment", "F1": "rating", "F2": "comment", "P1": "Rating (1-5)", "P2": "Additional comments", "BTN": "Submit", "CLR": "#a855f7", "MSG": "Survey submitted!", "EXTRA": ""}),
    ("invite", {"TITLE": "Invite", "DESC": "Invite form with email and role", "F1": "invite-email", "F2": "role", "P1": "Email to invite", "P2": "Role", "BTN": "Send Invite", "CLR": "#06b6d4", "MSG": "Invite sent!", "EXTRA": ""}),
    ("report", {"TITLE": "Report", "DESC": "Bug report form with title and description", "F1": "title", "F2": "description", "P1": "Issue Title", "P2": "Description", "BTN": "Report", "CLR": "#ef4444", "MSG": "Report filed!", "EXTRA": ""}),
    ("apply", {"TITLE": "Apply", "DESC": "Job application form with name and cover letter", "F1": "applicant", "F2": "cover", "P1": "Your Name", "P2": "Cover Letter", "BTN": "Apply", "CLR": "#3b82f6", "MSG": "Application sent!", "EXTRA": ""}),
    ("request", {"TITLE": "Request", "DESC": "Feature request form with title and reason", "F1": "feature", "F2": "reason", "P1": "Feature Name", "P2": "Why do you need this?", "BTN": "Request", "CLR": "#d946ef", "MSG": "Request logged!", "EXTRA": ""}),
]:
    ex(f"gen-form-{n}.naze", cfg["DESC"], fill(FORM_T, cfg))

# ─── Generator: Dashboards (grid + metrics) ──────────────────────────────────

DASH_T = """-- __DESC__
app "__TITLE__" {
  state __M1__ = __V1__
  state __M2__ = __V2__
  state __M3__ = __V3__
  state __M4__ = __V4__

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    grid columns: 2, gap: 16px {
      rect padding: 16px, color: __C1__, radius: 8px {
        text "__L1__" color: #64748b, font-size: 14px
        text "{__M1__}" font-size: 28px
      }
      rect padding: 16px, color: __C2__, radius: 8px {
        text "__L2__" color: #64748b, font-size: 14px
        text "{__M2__}" font-size: 28px
      }
      rect padding: 16px, color: __C3__, radius: 8px {
        text "__L3__" color: #64748b, font-size: 14px
        text "{__M3__}" font-size: 28px
      }
      rect padding: 16px, color: __C4__, radius: 8px {
        text "__L4__" color: #64748b, font-size: 14px
        text "{__M4__}" font-size: 28px
      }
    }
  }
}"""

for n, cfg in [
    ("sales", {"TITLE": "Sales", "DESC": "Sales dashboard with revenue metrics", "M1": "revenue", "V1": "52000", "L1": "Revenue", "C1": "#eff6ff", "M2": "deals", "V2": "128", "L2": "Deals", "C2": "#f0fdf4", "M3": "customers", "V3": "340", "L3": "Customers", "C3": "#fef3c7", "M4": "growth", "V4": "12", "L4": "Growth %", "C4": "#fce7f3"}),
    ("traffic", {"TITLE": "Traffic", "DESC": "Web traffic analytics dashboard", "M1": "visitors", "V1": "8400", "L1": "Visitors", "C1": "#f0f9ff", "M2": "page-views", "V2": "24000", "L2": "Page Views", "C2": "#ecfdf5", "M3": "bounce-rate", "V3": "35", "L3": "Bounce %", "C3": "#fefce8", "M4": "avg-time", "V4": "180", "L4": "Avg Time (s)", "C4": "#fdf2f8"}),
    ("fitness", {"TITLE": "Fitness", "DESC": "Fitness tracker dashboard", "M1": "steps", "V1": "8500", "L1": "Steps", "C1": "#ecfdf5", "M2": "calories", "V2": "2100", "L2": "Calories", "C2": "#fff7ed", "M3": "distance", "V3": "6", "L3": "Distance km", "C3": "#eff6ff", "M4": "active-min", "V4": "45", "L4": "Active Min", "C4": "#faf5ff"}),
    ("social", {"TITLE": "Social", "DESC": "Social media metrics dashboard", "M1": "followers", "V1": "12500", "L1": "Followers", "C1": "#f0f9ff", "M2": "likes", "V2": "3400", "L2": "Likes", "C2": "#fce7f3", "M3": "shares", "V3": "890", "L3": "Shares", "C3": "#ecfdf5", "M4": "comments", "V4": "456", "L4": "Comments", "C4": "#fef3c7"}),
    ("gaming", {"TITLE": "Game Stats", "DESC": "Gaming statistics dashboard", "M1": "score", "V1": "42000", "L1": "Score", "C1": "#faf5ff", "M2": "level", "V2": "28", "L2": "Level", "C2": "#eff6ff", "M3": "wins", "V3": "156", "L3": "Wins", "C3": "#ecfdf5", "M4": "streak", "V4": "12", "L4": "Win Streak", "C4": "#fefce8"}),
    ("education", {"TITLE": "Grades", "DESC": "Student grades dashboard", "M1": "gpa", "V1": "3", "L1": "GPA", "C1": "#eff6ff", "M2": "courses", "V2": "6", "L2": "Courses", "C2": "#f0fdf4", "M3": "credits", "V3": "18", "L3": "Credits", "C3": "#fff7ed", "M4": "rank", "V4": "15", "L4": "Class Rank", "C4": "#fce7f3"}),
    ("health", {"TITLE": "Health", "DESC": "Health metrics dashboard", "M1": "heart-rate", "V1": "72", "L1": "Heart Rate", "C1": "#fce7f3", "M2": "sleep-hrs", "V2": "7", "L2": "Sleep Hours", "C2": "#eff6ff", "M3": "water-ml", "V3": "2000", "L3": "Water (ml)", "C3": "#ecfdf5", "M4": "weight-kg", "V4": "68", "L4": "Weight (kg)", "C4": "#fefce8"}),
    ("energy", {"TITLE": "Energy", "DESC": "Home energy usage dashboard", "M1": "solar-kw", "V1": "4", "L1": "Solar (kW)", "C1": "#fefce8", "M2": "usage-kw", "V2": "6", "L2": "Usage (kW)", "C2": "#fce7f3", "M3": "cost", "V3": "85", "L3": "Cost ($)", "C3": "#eff6ff", "M4": "savings", "V4": "30", "L4": "Savings %", "C4": "#ecfdf5"}),
    ("project", {"TITLE": "Project", "DESC": "Project management dashboard", "M1": "tasks-done", "V1": "45", "L1": "Done", "C1": "#ecfdf5", "M2": "in-progress", "V2": "12", "L2": "In Progress", "C2": "#eff6ff", "M3": "bugs", "V3": "8", "L3": "Bugs", "C3": "#fce7f3", "M4": "velocity", "V4": "21", "L4": "Velocity", "C4": "#fef3c7"}),
    ("weather", {"TITLE": "Weather", "DESC": "Weather conditions dashboard", "M1": "temp", "V1": "22", "L1": "Temp (C)", "C1": "#fff7ed", "M2": "humidity", "V2": "65", "L2": "Humidity %", "C2": "#eff6ff", "M3": "wind", "V3": "15", "L3": "Wind (km/h)", "C3": "#ecfdf5", "M4": "uv-index", "V4": "6", "L4": "UV Index", "C4": "#fefce8"}),
    ("stocks", {"TITLE": "Portfolio", "DESC": "Stock portfolio dashboard", "M1": "total-value", "V1": "48500", "L1": "Total Value", "C1": "#ecfdf5", "M2": "daily-change", "V2": "250", "L2": "Daily +/-", "C2": "#eff6ff", "M3": "positions", "V3": "8", "L3": "Positions", "C3": "#fef3c7", "M4": "return-pct", "V4": "14", "L4": "Return %", "C4": "#fce7f3"}),
    ("hr", {"TITLE": "HR Dashboard", "DESC": "Human resources dashboard", "M1": "employees", "V1": "142", "L1": "Employees", "C1": "#eff6ff", "M2": "open-roles", "V2": "8", "L2": "Open Roles", "C2": "#fef3c7", "M3": "reviews", "V3": "24", "L3": "Reviews Due", "C3": "#fce7f3", "M4": "attendance", "V4": "96", "L4": "Attendance %", "C4": "#ecfdf5"}),
]:
    ex(f"gen-dash-{n}.naze", cfg["DESC"], fill(DASH_T, cfg))

# ─── Generator: Timers ───────────────────────────────────────────────────────

TIMER_T = """-- __DESC__
app "__TITLE__" {
  state __VAR__ = __INIT__
  state active = false

  timer tick: every __DUR__ {
    set __VAR__ = __VAR__ __OP__ 1
  }

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{__VAR__} __UNIT__" font-size: 48px, color: __CLR__

    row gap: 8px {
      rect width: 100px, height: 40px, color: #16a34a, radius: 8px {
        text "Start" color: #ffffff
        on click: set active = true
      }
      rect width: 100px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset" color: #ffffff
        on click: set __VAR__ = __INIT__
      }
    }
  }
}"""

for n, cfg in [
    ("stopwatch", {"TITLE": "Stopwatch", "DESC": "Stopwatch counting up in seconds", "VAR": "elapsed", "INIT": "0", "OP": "+", "DUR": "1s", "UNIT": "seconds", "CLR": "#2563eb"}),
    ("countdown-60", {"TITLE": "60s Timer", "DESC": "60-second countdown timer", "VAR": "remaining", "INIT": "60", "OP": "-", "DUR": "1s", "UNIT": "seconds left", "CLR": "#dc2626"}),
    ("countdown-5m", {"TITLE": "5 Min Timer", "DESC": "5-minute countdown timer", "VAR": "secs", "INIT": "300", "OP": "-", "DUR": "1s", "UNIT": "seconds", "CLR": "#f59e0b"}),
    ("clock-sec", {"TITLE": "Tick Clock", "DESC": "Clock ticking every second", "VAR": "ticks", "INIT": "0", "OP": "+", "DUR": "1s", "UNIT": "ticks", "CLR": "#6366f1"}),
    ("heartbeat", {"TITLE": "Heartbeat", "DESC": "Heartbeat monitor counting beats", "VAR": "beats", "INIT": "0", "OP": "+", "DUR": "1s", "UNIT": "bpm", "CLR": "#ef4444"}),
    ("quiz-timer", {"TITLE": "Quiz Timer", "DESC": "Quiz countdown with 30 second limit", "VAR": "time-left", "INIT": "30", "OP": "-", "DUR": "1s", "UNIT": "seconds", "CLR": "#8b5cf6"}),
    ("cooking", {"TITLE": "Cooking Timer", "DESC": "Cooking timer counting down minutes", "VAR": "mins", "INIT": "15", "OP": "-", "DUR": "1min", "UNIT": "minutes", "CLR": "#f97316"}),
    ("meditation", {"TITLE": "Meditate", "DESC": "Meditation timer counting breaths", "VAR": "breaths", "INIT": "0", "OP": "+", "DUR": "4s", "UNIT": "breaths", "CLR": "#14b8a6"}),
    ("workout", {"TITLE": "Workout", "DESC": "Workout interval timer", "VAR": "interval", "INIT": "45", "OP": "-", "DUR": "1s", "UNIT": "seconds", "CLR": "#22c55e"}),
    ("refresh", {"TITLE": "Auto Refresh", "DESC": "Auto-refresh counter every 5 seconds", "VAR": "refreshes", "INIT": "0", "OP": "+", "DUR": "5s", "UNIT": "refreshes", "CLR": "#0ea5e9"}),
    ("auction", {"TITLE": "Auction", "DESC": "Auction countdown timer", "VAR": "time-left", "INIT": "120", "OP": "-", "DUR": "1s", "UNIT": "seconds", "CLR": "#a855f7"}),
    ("slide", {"TITLE": "Slideshow", "DESC": "Slideshow auto-advance timer", "VAR": "slide", "INIT": "0", "OP": "+", "DUR": "3s", "UNIT": "slides shown", "CLR": "#ec4899"}),
]:
    ex(f"gen-timer-{n}.naze", cfg["DESC"], fill(TIMER_T, cfg))

# ─── Generator: Multi-page (navigation + links) ──────────────────────────────

NAV_T = """-- __DESC__
app "__TITLE__" {
  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__HOME_TEXT__"
    link "__L1__", to: "__P1__"
    link "__L2__", to: "__P2__"
    link "__L3__", to: "__P3__"
  }
}

page "__P1__" {
  column padding: 20px {
    heading "__H1__"
    text "__T1__"
    link "Back to Home", to: "/"
  }
}

page "__P2__" {
  column padding: 20px {
    heading "__H2__"
    text "__T2__"
    link "Back to Home", to: "/"
  }
}

page "__P3__" {
  column padding: 20px {
    heading "__H3__"
    text "__T3__"
    link "Back to Home", to: "/"
  }
}"""

for n, cfg in [
    ("blog", {"TITLE": "Blog", "DESC": "Blog with posts, archive, and about pages", "HOME_TEXT": "Welcome to my blog", "L1": "Posts", "P1": "/posts", "H1": "Blog Posts", "T1": "Latest articles", "L2": "Archive", "P2": "/archive", "H2": "Archive", "T2": "Past posts", "L3": "About", "P3": "/about", "H3": "About", "T3": "About the author"}),
    ("docs", {"TITLE": "Docs", "DESC": "Documentation site with guides and API reference", "HOME_TEXT": "Documentation hub", "L1": "Getting Started", "P1": "/start", "H1": "Getting Started", "T1": "Quick start guide", "L2": "API Reference", "P2": "/api", "H2": "API Reference", "T2": "Full API docs", "L3": "Examples", "P3": "/examples", "H3": "Examples", "T3": "Code examples"}),
    ("portfolio", {"TITLE": "Portfolio", "DESC": "Portfolio with projects, resume, and contact", "HOME_TEXT": "Welcome to my portfolio", "L1": "Projects", "P1": "/projects", "H1": "Projects", "T1": "My recent work", "L2": "Resume", "P2": "/resume", "H2": "Resume", "T2": "Experience and skills", "L3": "Contact", "P3": "/contact", "H3": "Contact", "T3": "Get in touch"}),
    ("store", {"TITLE": "Store", "DESC": "Online store with products, cart, and account", "HOME_TEXT": "Welcome to our store", "L1": "Products", "P1": "/products", "H1": "Products", "T1": "Browse our catalog", "L2": "Cart", "P2": "/cart", "H2": "Shopping Cart", "T2": "Your cart items", "L3": "Account", "P3": "/account", "H3": "My Account", "T3": "Account settings"}),
    ("company", {"TITLE": "Acme Corp", "DESC": "Company website with services, team, and careers", "HOME_TEXT": "Building the future", "L1": "Services", "P1": "/services", "H1": "Services", "T1": "What we offer", "L2": "Team", "P2": "/team", "H2": "Our Team", "T2": "Meet the team", "L3": "Careers", "P3": "/careers", "H3": "Careers", "T3": "Open positions"}),
    ("wiki", {"TITLE": "Wiki", "DESC": "Wiki with articles, categories, and recent changes", "HOME_TEXT": "Knowledge base", "L1": "Articles", "P1": "/articles", "H1": "Articles", "T1": "Browse articles", "L2": "Categories", "P2": "/categories", "H2": "Categories", "T2": "Topic categories", "L3": "Recent", "P3": "/recent", "H3": "Recent Changes", "T3": "Latest edits"}),
    ("news", {"TITLE": "News", "DESC": "News site with latest, trending, and categories", "HOME_TEXT": "Breaking news", "L1": "Latest", "P1": "/latest", "H1": "Latest News", "T1": "Most recent stories", "L2": "Trending", "P2": "/trending", "H2": "Trending", "T2": "Popular stories", "L3": "Categories", "P3": "/categories", "H3": "Categories", "T3": "News by topic"}),
    ("school", {"TITLE": "School", "DESC": "School portal with classes, grades, and schedule", "HOME_TEXT": "Student portal", "L1": "Classes", "P1": "/classes", "H1": "My Classes", "T1": "Current enrollment", "L2": "Grades", "P2": "/grades", "H2": "Grades", "T2": "Academic performance", "L3": "Schedule", "P3": "/schedule", "H3": "Schedule", "T3": "Weekly timetable"}),
    ("music", {"TITLE": "Music", "DESC": "Music app with library, playlists, and discover", "HOME_TEXT": "Your music", "L1": "Library", "P1": "/library", "H1": "My Library", "T1": "Your saved music", "L2": "Playlists", "P2": "/playlists", "H2": "Playlists", "T2": "Your playlists", "L3": "Discover", "P3": "/discover", "H3": "Discover", "T3": "Find new music"}),
    ("fitness-app", {"TITLE": "FitTrack", "DESC": "Fitness app with workouts, nutrition, and progress", "HOME_TEXT": "Your fitness journey", "L1": "Workouts", "P1": "/workouts", "H1": "Workouts", "T1": "Today's exercises", "L2": "Nutrition", "P2": "/nutrition", "H2": "Nutrition", "T2": "Meal tracking", "L3": "Progress", "P3": "/progress", "H3": "Progress", "T3": "Your stats"}),
]:
    ex(f"gen-nav-{n}.naze", cfg["DESC"], fill(NAV_T, cfg))

# ─── Generator: Toggles (match + state switching) ────────────────────────────

TOGGLE_T = """-- __DESC__
app "__TITLE__" {
  state __VAR__ = "__INIT__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    match __VAR__ {
      "__O1__": __B1__
      "__O2__": __B2__
      "__O3__": __B3__
      _: text "Select an option" color: #64748b
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: __C1__, radius: 4px {
        text "__O1__" color: #ffffff
        on click: set __VAR__ = "__O1__"
      }
      rect width: 100px, height: 36px, color: __C2__, radius: 4px {
        text "__O2__" color: #ffffff
        on click: set __VAR__ = "__O2__"
      }
      rect width: 100px, height: 36px, color: __C3__, radius: 4px {
        text "__O3__" color: #ffffff
        on click: set __VAR__ = "__O3__"
      }
    }
  }
}"""

for n, cfg in [
    ("tabs", {"TITLE": "Tabs", "DESC": "Tab navigation with content switching", "VAR": "tab", "INIT": "home", "O1": "home", "O2": "settings", "O3": "help", "B1": 'text "Welcome home" font-size: 18px', "B2": 'text "App settings" font-size: 18px', "B3": 'text "Help & FAQ" font-size: 18px', "C1": "#2563eb", "C2": "#16a34a", "C3": "#f59e0b"}),
    ("view-mode", {"TITLE": "View Mode", "DESC": "Switch between list, grid, and card views", "VAR": "view", "INIT": "list", "O1": "list", "O2": "grid", "O3": "card", "B1": 'text "List view active" font-size: 18px', "B2": 'text "Grid view active" font-size: 18px', "B3": 'text "Card view active" font-size: 18px', "C1": "#6366f1", "C2": "#ec4899", "C3": "#14b8a6"}),
    ("sort", {"TITLE": "Sort By", "DESC": "Sort option selector with name, date, and size", "VAR": "sort-by", "INIT": "name", "O1": "name", "O2": "date", "O3": "size", "B1": 'text "Sorted by name" color: #2563eb', "B2": 'text "Sorted by date" color: #2563eb', "B3": 'text "Sorted by size" color: #2563eb', "C1": "#64748b", "C2": "#64748b", "C3": "#64748b"}),
    ("status", {"TITLE": "Status", "DESC": "Status selector with active, paused, and stopped states", "VAR": "status", "INIT": "active", "O1": "active", "O2": "paused", "O3": "stopped", "B1": 'text "System active" color: #16a34a, font-size: 20px', "B2": 'text "System paused" color: #f59e0b, font-size: 20px', "B3": 'text "System stopped" color: #dc2626, font-size: 20px', "C1": "#16a34a", "C2": "#f59e0b", "C3": "#dc2626"}),
    ("lang", {"TITLE": "Language", "DESC": "Language selector with English, Spanish, and French", "VAR": "lang", "INIT": "en", "O1": "en", "O2": "es", "O3": "fr", "B1": 'text "Hello, World!" font-size: 20px', "B2": 'text "Hola, Mundo!" font-size: 20px', "B3": 'text "Bonjour, le Monde!" font-size: 20px', "C1": "#3b82f6", "C2": "#ef4444", "C3": "#1d4ed8"}),
    ("size", {"TITLE": "Size", "DESC": "Size picker with small, medium, and large options", "VAR": "size", "INIT": "medium", "O1": "small", "O2": "medium", "O3": "large", "B1": 'text "Small selected" font-size: 14px', "B2": 'text "Medium selected" font-size: 18px', "B3": 'text "Large selected" font-size: 24px', "C1": "#94a3b8", "C2": "#64748b", "C3": "#334155"}),
    ("layout", {"TITLE": "Layout", "DESC": "Layout mode with compact, normal, and wide options", "VAR": "layout", "INIT": "normal", "O1": "compact", "O2": "normal", "O3": "wide", "B1": 'text "Compact layout" color: #64748b', "B2": 'text "Normal layout" color: #64748b', "B3": 'text "Wide layout" color: #64748b', "C1": "#0ea5e9", "C2": "#2563eb", "C3": "#7c3aed"}),
    ("speed", {"TITLE": "Speed", "DESC": "Playback speed selector with slow, normal, and fast", "VAR": "speed", "INIT": "normal", "O1": "slow", "O2": "normal", "O3": "fast", "B1": 'text "0.5x speed" font-size: 18px', "B2": 'text "1x speed" font-size: 18px', "B3": 'text "2x speed" font-size: 18px', "C1": "#64748b", "C2": "#2563eb", "C3": "#ef4444"}),
    ("priority", {"TITLE": "Priority", "DESC": "Priority selector with low, medium, and high", "VAR": "priority", "INIT": "medium", "O1": "low", "O2": "medium", "O3": "high", "B1": 'text "Low priority" color: #16a34a', "B2": 'text "Medium priority" color: #f59e0b', "B3": 'text "High priority" color: #dc2626', "C1": "#16a34a", "C2": "#f59e0b", "C3": "#dc2626"}),
    ("difficulty", {"TITLE": "Difficulty", "DESC": "Difficulty picker for easy, normal, and hard modes", "VAR": "difficulty", "INIT": "normal", "O1": "easy", "O2": "normal", "O3": "hard", "B1": 'text "Easy mode" color: #16a34a, font-size: 20px', "B2": 'text "Normal mode" color: #2563eb, font-size: 20px', "B3": 'text "Hard mode" color: #dc2626, font-size: 20px', "C1": "#16a34a", "C2": "#2563eb", "C3": "#dc2626"}),
]:
    ex(f"gen-toggle-{n}.naze", cfg["DESC"], fill(TOGGLE_T, cfg))

# ─── Generator: Data fetch + display ─────────────────────────────────────────

FETCH_T = """-- __DESC__
app "__TITLE__" {
  data __NAME__: fetch "__URL__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    if __NAME__.loading {
      text "Loading __LABEL__..." color: #64748b
    }

    if __NAME__.error {
      text "Failed to load __LABEL__" color: #dc2626
    }

    if __NAME__.data {
      text "__LABEL__ loaded successfully" color: #16a34a
      each __IT__ in __NAME__.data {
        row padding: 8px, color: __BG__, radius: 4px {
          text "{__IT__.__FIELD__}"
        }
      }
    }
  }
}"""

for n, cfg in [
    ("posts", {"TITLE": "Posts", "DESC": "Blog posts fetched from API", "NAME": "posts", "URL": "/api/posts", "LABEL": "posts", "IT": "post", "FIELD": "title", "BG": "#f3f4f6"}),
    ("comments", {"TITLE": "Comments", "DESC": "Comments fetched from API", "NAME": "comments", "URL": "/api/comments", "LABEL": "comments", "IT": "comment", "FIELD": "text", "BG": "#f0f9ff"}),
    ("photos", {"TITLE": "Photos", "DESC": "Photo gallery from API", "NAME": "photos", "URL": "/api/photos", "LABEL": "photos", "IT": "photo", "FIELD": "title", "BG": "#fdf2f8"}),
    ("repos", {"TITLE": "Repos", "DESC": "GitHub repositories from API", "NAME": "repos", "URL": "/api/repos", "LABEL": "repos", "IT": "repo", "FIELD": "name", "BG": "#f8fafc"}),
    ("todos", {"TITLE": "Todos", "DESC": "Todo items fetched from API", "NAME": "todos", "URL": "/api/todos", "LABEL": "todos", "IT": "todo", "FIELD": "title", "BG": "#ecfdf5"}),
    ("users-api", {"TITLE": "API Users", "DESC": "User list from remote API", "NAME": "users", "URL": "https://api.example.com/users", "LABEL": "users", "IT": "user", "FIELD": "name", "BG": "#eff6ff"}),
    ("weather-api", {"TITLE": "Forecast", "DESC": "Weather forecast from API", "NAME": "forecast", "URL": "/api/weather", "LABEL": "forecast", "IT": "day", "FIELD": "summary", "BG": "#fff7ed"}),
    ("news-feed", {"TITLE": "News Feed", "DESC": "News articles from API", "NAME": "articles", "URL": "/api/news", "LABEL": "articles", "IT": "article", "FIELD": "headline", "BG": "#fefce8"}),
    ("notifications", {"TITLE": "Notifications", "DESC": "Notification feed from API", "NAME": "notifs", "URL": "/api/notifications", "LABEL": "notifications", "IT": "notif", "FIELD": "message", "BG": "#faf5ff"}),
    ("products-api", {"TITLE": "Catalog", "DESC": "Product catalog from API", "NAME": "catalog", "URL": "/api/products", "LABEL": "products", "IT": "item", "FIELD": "name", "BG": "#f0fdf4"}),
    ("messages", {"TITLE": "Messages", "DESC": "Message inbox from API", "NAME": "inbox", "URL": "/api/messages", "LABEL": "messages", "IT": "msg", "FIELD": "subject", "BG": "#fce7f3"}),
    ("events-api", {"TITLE": "Events", "DESC": "Upcoming events from API", "NAME": "events", "URL": "/api/events", "LABEL": "events", "IT": "evt", "FIELD": "title", "BG": "#ecfeff"}),
]:
    ex(f"gen-fetch-{n}.naze", cfg["DESC"], fill(FETCH_T, cfg))

# ─── Generator: Interactive apps (click + state) ─────────────────────────────

INTERACT_T = """-- __DESC__
app "__TITLE__" {
  state __S1__ = __V1__
  state __S2__ = __V2__

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{__S1__}" font-size: 24px, color: __CLR__
    text "{__S2__}" color: #64748b

    row gap: 8px {
      rect width: __BW__px, height: 40px, color: __CLR__, radius: 8px {
        text "__B1__" color: #ffffff
        on click: set __S1__ = __A1__
      }
      rect width: __BW__px, height: 40px, color: #64748b, radius: 8px {
        text "__B2__" color: #ffffff
        on click: set __S2__ = __A2__
      }
      rect width: __BW__px, height: 40px, color: #e2e8f0, radius: 8px {
        text "Reset"
        on click: set __S1__ = __V1__
      }
    }
  }
}"""

for n, cfg in [
    ("like", {"TITLE": "Like Button", "DESC": "Like counter with toggle", "S1": "likes", "V1": "0", "S2": "status", "V2": '"none"', "B1": "Like", "A1": "likes + 1", "B2": "Dislike", "A2": '"disliked"', "CLR": "#ef4444", "BW": "80"}),
    ("volume", {"TITLE": "Volume", "DESC": "Volume control with up and down", "S1": "volume", "V1": "50", "S2": "muted", "V2": "false", "B1": "Up", "A1": "volume + 10", "B2": "Down", "A2": "true", "CLR": "#2563eb", "BW": "80"}),
    ("zoom", {"TITLE": "Zoom", "DESC": "Zoom level control", "S1": "zoom", "V1": "100", "S2": "mode", "V2": '"fit"', "B1": "Zoom In", "A1": "zoom + 25", "B2": "Zoom Out", "A2": '"manual"', "CLR": "#6366f1", "BW": "100"}),
    ("rating", {"TITLE": "Rating", "DESC": "Star rating incrementer", "S1": "stars", "V1": "0", "S2": "rated", "V2": "false", "B1": "Add Star", "A1": "stars + 1", "B2": "Submit", "A2": "true", "CLR": "#f59e0b", "BW": "100"}),
    ("brightness", {"TITLE": "Brightness", "DESC": "Brightness slider control", "S1": "brightness", "V1": "50", "S2": "auto", "V2": "false", "B1": "Brighter", "A1": "brightness + 10", "B2": "Auto", "A2": "true", "CLR": "#f97316", "BW": "100"}),
    ("font-size", {"TITLE": "Font Size", "DESC": "Font size adjuster", "S1": "size", "V1": "16", "S2": "unit", "V2": '"px"', "B1": "Bigger", "A1": "size + 2", "B2": "Smaller", "A2": '"px"', "CLR": "#8b5cf6", "BW": "80"}),
    ("quantity", {"TITLE": "Quantity", "DESC": "Quantity selector for shopping", "S1": "qty", "V1": "1", "S2": "added", "V2": "false", "B1": "More", "A1": "qty + 1", "B2": "Add Cart", "A2": "true", "CLR": "#16a34a", "BW": "100"}),
    ("score-game", {"TITLE": "Score", "DESC": "Game score tracker", "S1": "points", "V1": "0", "S2": "combo", "V2": "0", "B1": "Score!", "A1": "points + 10", "B2": "Combo", "A2": "combo + 1", "CLR": "#ec4899", "BW": "80"}),
    ("temp-ctrl", {"TITLE": "Thermostat", "DESC": "Temperature controller", "S1": "target", "V1": "20", "S2": "mode", "V2": '"heat"', "B1": "Warmer", "A1": "target + 1", "B2": "Cooler", "A2": '"cool"', "CLR": "#ef4444", "BW": "80"}),
    ("progress", {"TITLE": "Progress", "DESC": "Progress tracker with increment", "S1": "done", "V1": "0", "S2": "total-items", "V2": "10", "B1": "Complete", "A1": "done + 1", "B2": "Skip", "A2": "total-items", "CLR": "#14b8a6", "BW": "100"}),
    ("bid", {"TITLE": "Bidding", "DESC": "Auction bid incrementer", "S1": "bid", "V1": "100", "S2": "bids", "V2": "0", "B1": "Bid +10", "A1": "bid + 10", "B2": "Bid +50", "A2": "bids + 1", "CLR": "#a855f7", "BW": "80"}),
    ("donate", {"TITLE": "Donate", "DESC": "Donation amount selector", "S1": "amount", "V1": "10", "S2": "donated", "V2": "false", "B1": "Add $5", "A1": "amount + 5", "B2": "Donate", "A2": "true", "CLR": "#22c55e", "BW": "80"}),
]:
    ex(f"gen-interact-{n}.naze", cfg["DESC"], fill(INTERACT_T, cfg))

# ─── Generator: Theme variations ─────────────────────────────────────────────

THEME_T = """-- __DESC__
theme __TNAME__ {
  colors {
    primary: __P__
    secondary: __S__
    bg: __BG__
    text-color: __FG__
    accent: __AC__
  }
}

app "__TITLE__" {
  column padding: 20px, gap: 16px, color: theme.colors.bg {
    heading "__TITLE__" color: theme.colors.text-color

    rect padding: 16px, color: theme.colors.primary, radius: 8px {
      text "Primary" color: #ffffff
    }
    rect padding: 16px, color: theme.colors.secondary, radius: 8px {
      text "Secondary" color: #ffffff
    }
    rect padding: 16px, color: theme.colors.accent, radius: 8px {
      text "Accent" color: #ffffff
    }
  }
}"""

for n, cfg in [
    ("ocean", {"TITLE": "Ocean Theme", "DESC": "Ocean-inspired blue color scheme", "TNAME": "ocean", "P": "#0077b6", "S": "#00b4d8", "BG": "#caf0f8", "FG": "#03045e", "AC": "#0096c7"}),
    ("forest", {"TITLE": "Forest Theme", "DESC": "Forest-inspired green color scheme", "TNAME": "forest", "P": "#2d6a4f", "S": "#40916c", "BG": "#d8f3dc", "FG": "#1b4332", "AC": "#52b788"}),
    ("sunset", {"TITLE": "Sunset Theme", "DESC": "Warm sunset-inspired color scheme", "TNAME": "sunset", "P": "#e63946", "S": "#f4a261", "BG": "#fff1e6", "FG": "#1d3557", "AC": "#e76f51"}),
    ("midnight", {"TITLE": "Midnight Theme", "DESC": "Dark midnight color scheme", "TNAME": "midnight", "P": "#7209b7", "S": "#3a0ca3", "BG": "#10002b", "FG": "#e0aaff", "AC": "#560bad"}),
    ("pastel", {"TITLE": "Pastel Theme", "DESC": "Soft pastel color scheme", "TNAME": "pastel", "P": "#cdb4db", "S": "#ffc8dd", "BG": "#f8edeb", "FG": "#6d6875", "AC": "#a2d2ff"}),
    ("earth", {"TITLE": "Earth Theme", "DESC": "Earthy warm tone color scheme", "TNAME": "earth", "P": "#bc6c25", "S": "#dda15e", "BG": "#fefae0", "FG": "#283618", "AC": "#606c38"}),
    ("arctic", {"TITLE": "Arctic Theme", "DESC": "Cool arctic color scheme", "TNAME": "arctic", "P": "#48cae4", "S": "#90e0ef", "BG": "#f0f8ff", "FG": "#023e8a", "AC": "#00b4d8"}),
    ("lavender", {"TITLE": "Lavender Theme", "DESC": "Lavender purple color scheme", "TNAME": "lavender", "P": "#7b2cbf", "S": "#9d4edd", "BG": "#f3e8ff", "FG": "#3c096c", "AC": "#c77dff"}),
    ("coral", {"TITLE": "Coral Theme", "DESC": "Coral and pink color scheme", "TNAME": "coral", "P": "#ff6b6b", "S": "#ffa8a8", "BG": "#fff5f5", "FG": "#c92a2a", "AC": "#ff8787"}),
    ("slate", {"TITLE": "Slate Theme", "DESC": "Professional slate gray color scheme", "TNAME": "slate", "P": "#475569", "S": "#64748b", "BG": "#f8fafc", "FG": "#0f172a", "AC": "#94a3b8"}),
]:
    ex(f"gen-theme-{n}.naze", cfg["DESC"], fill(THEME_T, cfg))

# ─── Generator: App archetypes (unique real-world apps) ──────────────────────

# Chat interface
ex("gen-app-chat.naze", "Chat interface with message input and send button",
   """-- Chat interface
app "Chat" {
  state message = ""
  state messages = []

  column padding: 20px, gap: 16px {
    heading "Chat"

    scroll height: 300px {
      column gap: 4px {
        each msg in messages {
          rect padding: 8px, color: #f3f4f6, radius: 8px {
            text "{msg}"
          }
        }
      }
    }

    row gap: 8px {
      input bind: message, placeholder: "Type a message..."
      rect width: 80px, height: 40px, color: #2563eb, radius: 8px {
        text "Send" color: #ffffff
        on click: append message to messages
      }
    }
  }
}""")

# Todo list
ex("gen-app-todo.naze", "Todo list with add, complete markers, and count",
   """-- Todo app
app "Todos" {
  state task = ""
  state tasks = []
  computed pending = tasks | count

  column padding: 20px, gap: 16px {
    heading "Todo List"
    text "{pending} tasks" color: #64748b

    row gap: 8px {
      input bind: task, placeholder: "New task..."
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Add" color: #ffffff
        on click: append task to tasks
      }
    }

    each t in tasks {
      row padding: 8px, color: #f3f4f6, radius: 4px, gap: 8px {
        text "{t}"
        rect width: 60px, height: 28px, color: #ef4444, radius: 4px {
          text "Del" color: #ffffff
          on click: remove t from tasks
        }
      }
    }
  }
}""")

# Notes app
ex("gen-app-notes.naze", "Simple notes app with title, body, and save",
   """-- Notes app
app "Notes" {
  state title = ""
  state body = ""
  state saved = false

  column padding: 20px, gap: 16px {
    heading "New Note"
    input bind: title, placeholder: "Note title..."
    textarea bind: body, placeholder: "Write your note..."
    rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
      text "Save" color: #ffffff
      on click: set saved = true
    }

    if saved {
      text "Note saved!" color: #16a34a
    }
  }
}""")

# Calculator
ex("gen-app-calculator.naze", "Simple calculator with display and operation buttons",
   """-- Calculator
app "Calculator" {
  state display = 0
  state memory = 0

  column padding: 20px, gap: 8px {
    heading "Calculator"
    text "{display}" font-size: 36px, color: #1e293b

    grid columns: 4, gap: 4px {
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "7"
        on click: set display = 7
      }
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "8"
        on click: set display = 8
      }
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "9"
        on click: set display = 9
      }
      rect width: 60px, height: 50px, color: #f59e0b, radius: 4px {
        text "C" color: #ffffff
        on click: set display = 0
      }
    }

    grid columns: 4, gap: 4px {
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "4"
        on click: set display = 4
      }
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "5"
        on click: set display = 5
      }
      rect width: 60px, height: 50px, color: #e2e8f0, radius: 4px {
        text "6"
        on click: set display = 6
      }
      rect width: 60px, height: 50px, color: #2563eb, radius: 4px {
        text "+" color: #ffffff
        on click: set memory = display
      }
    }
  }
}""")

# Kanban board
ex("gen-app-kanban.naze", "Kanban board with todo, doing, and done columns",
   """-- Kanban board
app "Kanban" {
  state todo-items = ["Design UI", "Write tests"]
  state doing-items = ["Build API"]
  state done-items = ["Setup repo"]

  column padding: 20px, gap: 16px {
    heading "Kanban Board"

    row gap: 16px {
      column gap: 8px {
        text "Todo" font-weight: bold, color: #dc2626
        each item in todo-items {
          rect padding: 8px, color: #fef2f2, radius: 4px {
            text "{item}"
          }
        }
      }

      column gap: 8px {
        text "Doing" font-weight: bold, color: #f59e0b
        each item in doing-items {
          rect padding: 8px, color: #fffbeb, radius: 4px {
            text "{item}"
          }
        }
      }

      column gap: 8px {
        text "Done" font-weight: bold, color: #16a34a
        each item in done-items {
          rect padding: 8px, color: #f0fdf4, radius: 4px {
            text "{item}"
          }
        }
      }
    }
  }
}""")

# Music player
ex("gen-app-player.naze", "Music player with track info, progress, and controls",
   """-- Music player
app "Player" {
  state track = "Bohemian Rhapsody"
  state artist = "Queen"
  state playing = false
  state position = 0

  timer playback: every 1s {
    set position = position + 1
  }

  column padding: 20px, gap: 16px {
    heading "Now Playing"
    text "{track}" font-size: 24px
    text "{artist}" color: #64748b
    text "{position}s elapsed" color: #94a3b8

    row gap: 8px {
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Play" color: #ffffff
        on click: set playing = true
      }
      rect width: 80px, height: 40px, color: #64748b, radius: 8px {
        text "Pause" color: #ffffff
        on click: set playing = false
      }
      rect width: 80px, height: 40px, color: #ef4444, radius: 8px {
        text "Stop" color: #ffffff
        on click: set position = 0
      }
    }
  }
}""")

# Recipe viewer
ex("gen-app-recipe.naze", "Recipe viewer with ingredients list and steps",
   """-- Recipe viewer
app "Recipe" {
  state servings = 4
  state ingredients = [{name: "Flour", amount: "2 cups"}, {name: "Sugar", amount: "1 cup"}, {name: "Eggs", amount: "3"}]

  computed ingredient-count = ingredients | count

  column padding: 20px, gap: 16px {
    heading "Chocolate Cake"
    text "Servings: {servings}" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "-" color: #ffffff
        on click: set servings = servings - 1
      }
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "+" color: #ffffff
        on click: set servings = servings + 1
      }
    }

    text "Ingredients ({ingredient-count}):" font-weight: bold

    each ing in ingredients {
      row padding: 4px, gap: 8px {
        text "{ing.amount}" color: #2563eb
        text "{ing.name}"
      }
    }
  }
}""")

# Polling/voting
ex("gen-app-poll.naze", "Poll with vote buttons and tallies",
   """-- Poll
app "Poll" {
  state option-a = 0
  state option-b = 0
  state option-c = 0
  computed total-votes = option-a + option-b + option-c

  column padding: 20px, gap: 16px {
    heading "What is your favorite?"
    text "{total-votes} total votes" color: #64748b

    column gap: 8px {
      row gap: 8px {
        rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
          text "Pizza ({option-a})" color: #ffffff
          on click: set option-a = option-a + 1
        }
      }
      row gap: 8px {
        rect width: 120px, height: 40px, color: #16a34a, radius: 8px {
          text "Sushi ({option-b})" color: #ffffff
          on click: set option-b = option-b + 1
        }
      }
      row gap: 8px {
        rect width: 120px, height: 40px, color: #f59e0b, radius: 8px {
          text "Tacos ({option-c})" color: #ffffff
          on click: set option-c = option-c + 1
        }
      }
    }
  }
}""")

# Bookmark manager
ex("gen-app-bookmarks.naze", "Bookmark manager with URL input and saved list",
   """-- Bookmarks
app "Bookmarks" {
  state url = ""
  state bookmarks = ["https://example.com", "https://github.com"]
  computed count = bookmarks | count

  column padding: 20px, gap: 16px {
    heading "Bookmarks"
    text "{count} saved" color: #64748b

    row gap: 8px {
      input bind: url, placeholder: "Add URL..."
      rect width: 80px, height: 40px, color: #2563eb, radius: 8px {
        text "Save" color: #ffffff
        on click: append url to bookmarks
      }
    }

    each bm in bookmarks {
      row padding: 8px, color: #f3f4f6, radius: 4px, gap: 8px {
        text "{bm}" color: #2563eb
        rect width: 60px, height: 28px, color: #ef4444, radius: 4px {
          text "Del" color: #ffffff
          on click: remove bm from bookmarks
        }
      }
    }
  }
}""")

# Habit tracker
ex("gen-app-habits.naze", "Daily habit tracker with streak counter",
   """-- Habit tracker
app "Habits" {
  state habits = [{name: "Exercise", streak: "5"}, {name: "Reading", streak: "12"}, {name: "Meditate", streak: "3"}]
  state new-habit = ""

  column padding: 20px, gap: 16px {
    heading "Daily Habits"

    row gap: 8px {
      input bind: new-habit, placeholder: "New habit..."
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Add" color: #ffffff
        on click: set new-habit = ""
      }
    }

    each habit in habits {
      row padding: 12px, color: #f0fdf4, radius: 8px, gap: 12px {
        text "{habit.name}" font-weight: bold
        text "{habit.streak} day streak" color: #16a34a
      }
    }
  }
}""")

# Inventory tracker
ex("gen-app-inventory.naze", "Inventory manager with stock levels and alerts",
   """-- Inventory
app "Inventory" {
  state products = [{name: "Widget A", stock: "150"}, {name: "Widget B", stock: "3"}, {name: "Widget C", stock: "45"}]
  computed total-items = products | count

  column padding: 20px, gap: 16px {
    heading "Inventory"
    text "{total-items} products tracked" color: #64748b

    each p in products | sort-by name {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        text "{p.name}" font-weight: bold
        text "Stock: {p.stock}" color: #64748b
      }
    }
  }
}""")

# Flashcard app
ex("gen-app-flashcards.naze", "Flashcard study app with flip and next",
   """-- Flashcards
app "Flashcards" {
  state showing = "front"
  state card-index = 0
  state front = "What is the capital of France?"
  state back = "Paris"

  column padding: 20px, gap: 16px {
    heading "Study Cards"

    rect width: 300px, height: 200px, color: #eff6ff, radius: 12px, padding: 20px {
      match showing {
        "front": text "{front}" font-size: 18px
        "back": text "{back}" font-size: 24px, color: #2563eb
        _: text "Error"
      }
    }

    row gap: 8px {
      rect width: 80px, height: 40px, color: #6366f1, radius: 8px {
        text "Flip" color: #ffffff
        on click: set showing = "back"
      }
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Next" color: #ffffff
        on click: set showing = "front"
      }
    }
  }
}""")

# Password generator
ex("gen-app-password.naze", "Password generator with length control",
   """-- Password generator
app "Passwords" {
  state length = 16
  state generated = ""
  state copied = false

  column padding: 20px, gap: 16px {
    heading "Password Generator"
    text "Length: {length}" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 36px, color: #e2e8f0, radius: 4px {
        text "-"
        on click: set length = length - 1
      }
      rect width: 60px, height: 36px, color: #e2e8f0, radius: 4px {
        text "+"
        on click: set length = length + 1
      }
    }

    rect width: 150px, height: 40px, color: #7c3aed, radius: 8px {
      text "Generate" color: #ffffff
      on click: set generated = "password"
    }

    if generated {
      row gap: 8px {
        text "{generated}" font-size: 18px
        rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
          text "Copy" color: #ffffff
          on click: copy generated
        }
      }
    }
  }
}""")

# Timer with laps
ex("gen-app-laps.naze", "Lap timer with elapsed time and lap list",
   """-- Lap timer
app "Lap Timer" {
  state elapsed = 0
  state laps = []

  timer tick: every 1s {
    set elapsed = elapsed + 1
  }

  column padding: 20px, gap: 16px {
    heading "Lap Timer"
    text "{elapsed} seconds" font-size: 36px, color: #2563eb

    row gap: 8px {
      rect width: 80px, height: 40px, color: #f59e0b, radius: 8px {
        text "Lap" color: #ffffff
        on click: append elapsed to laps
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset" color: #ffffff
        on click: set elapsed = 0
      }
    }

    each lap in laps {
      text "Lap: {lap}s" color: #64748b
    }
  }
}""")

# FAQ accordion
ex("gen-app-faq.naze", "FAQ section with expandable question-answer pairs",
   """-- FAQ
app "FAQ" {
  state active = "none"

  column padding: 20px, gap: 8px {
    heading "Frequently Asked Questions"

    rect padding: 12px, color: #f3f4f6, radius: 4px {
      text "What is Naze?" font-weight: bold
      on click: set active = "q1"
    }

    if active == "q1" {
      rect padding: 12px, color: #eff6ff {
        text "Naze is a declarative UI language that compiles to WASM."
      }
    }

    rect padding: 12px, color: #f3f4f6, radius: 4px {
      text "How does it work?" font-weight: bold
      on click: set active = "q2"
    }

    if active == "q2" {
      rect padding: 12px, color: #eff6ff {
        text "It parses .naze files into an AST, compiles to IR, and renders via Canvas2D."
      }
    }

    rect padding: 12px, color: #f3f4f6, radius: 4px {
      text "Is it open source?" font-weight: bold
      on click: set active = "q3"
    }

    if active == "q3" {
      rect padding: 12px, color: #eff6ff {
        text "Yes, Naze is fully open source."
      }
    }
  }
}""")

# Color palette
ex("gen-app-palette.naze", "Color palette display with hex values",
   """-- Color palette
app "Palette" {
  state selected = "blue"

  column padding: 20px, gap: 16px {
    heading "Color Palette"
    text "Selected: {selected}" color: #64748b

    grid columns: 4, gap: 8px {
      rect width: 60px, height: 60px, color: #ef4444, radius: 8px {
        on click: set selected = "red"
      }
      rect width: 60px, height: 60px, color: #f59e0b, radius: 8px {
        on click: set selected = "yellow"
      }
      rect width: 60px, height: 60px, color: #22c55e, radius: 8px {
        on click: set selected = "green"
      }
      rect width: 60px, height: 60px, color: #3b82f6, radius: 8px {
        on click: set selected = "blue"
      }
      rect width: 60px, height: 60px, color: #8b5cf6, radius: 8px {
        on click: set selected = "purple"
      }
      rect width: 60px, height: 60px, color: #ec4899, radius: 8px {
        on click: set selected = "pink"
      }
      rect width: 60px, height: 60px, color: #14b8a6, radius: 8px {
        on click: set selected = "teal"
      }
      rect width: 60px, height: 60px, color: #f97316, radius: 8px {
        on click: set selected = "orange"
      }
    }
  }
}""")

# Pricing page
ex("gen-app-pricing.naze", "Pricing page with three plan tiers",
   """-- Pricing page
app "Pricing" {
  state selected-plan = "none"

  column padding: 20px, gap: 16px {
    heading "Choose a Plan"

    grid columns: 3, gap: 16px {
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Basic" font-weight: bold, font-size: 18px
          text "$9/mo" font-size: 24px, color: #2563eb
          text "5 projects" color: #64748b
          rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
            text "Select" color: #ffffff
            on click: set selected-plan = "basic"
          }
        }
      }
      rect padding: 20px, color: #eff6ff, radius: 12px {
        column gap: 8px {
          text "Pro" font-weight: bold, font-size: 18px
          text "$29/mo" font-size: 24px, color: #2563eb
          text "Unlimited" color: #64748b
          rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
            text "Select" color: #ffffff
            on click: set selected-plan = "pro"
          }
        }
      }
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Enterprise" font-weight: bold, font-size: 18px
          text "Custom" font-size: 24px, color: #2563eb
          text "Dedicated" color: #64748b
          rect width: 100px, height: 36px, color: #64748b, radius: 8px {
            text "Contact" color: #ffffff
            on click: set selected-plan = "enterprise"
          }
        }
      }
    }

    if selected-plan != "none" {
      text "Selected: {selected-plan}" color: #16a34a, font-size: 18px
    }
  }
}""")

# Status page
ex("gen-app-status.naze", "Service status page with system health indicators",
   """-- Status page
app "Status" {
  state api-status = "operational"
  state db-status = "operational"
  state cdn-status = "degraded"

  column padding: 20px, gap: 16px {
    heading "System Status"

    column gap: 8px {
      row padding: 12px, color: #f0fdf4, radius: 4px, gap: 8px {
        text "API" font-weight: bold
        text "{api-status}" color: #16a34a
      }
      row padding: 12px, color: #f0fdf4, radius: 4px, gap: 8px {
        text "Database" font-weight: bold
        text "{db-status}" color: #16a34a
      }
      row padding: 12px, color: #fefce8, radius: 4px, gap: 8px {
        text "CDN" font-weight: bold
        text "{cdn-status}" color: #f59e0b
      }
    }
  }
}""")

# Changelog
ex("gen-app-changelog.naze", "Changelog with version entries and dates",
   """-- Changelog
app "Changelog" {
  state entries = [{version: "2.1.0", date: "2024-01-15", note: "Added dark mode"}, {version: "2.0.0", date: "2024-01-01", note: "Major redesign"}, {version: "1.5.0", date: "2023-12-15", note: "Performance improvements"}]

  column padding: 20px, gap: 16px {
    heading "Changelog"

    each entry in entries {
      column padding: 12px, gap: 4px {
        row gap: 8px {
          text "v{entry.version}" font-weight: bold, color: #2563eb
          text "{entry.date}" color: #94a3b8
        }
        text "{entry.note}" color: #374151
        separator
      }
    }
  }
}""")

# Leaderboard
ex("gen-app-leaderboard.naze", "Game leaderboard with ranked players",
   """-- Leaderboard
app "Leaderboard" {
  state players = [{name: "Alice", score: "2450"}, {name: "Bob", score: "2100"}, {name: "Carol", score: "1950"}, {name: "Dan", score: "1800"}]

  column padding: 20px, gap: 16px {
    heading "Leaderboard"

    each player in players | sort-by score {
      row padding: 12px, color: #fef3c7, radius: 4px, gap: 16px {
        text "{player.name}" font-weight: bold
        text "{player.score} pts" color: #92400e
      }
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# GENERATORS: Diverse domain patterns (75 examples)
# Categories: E-commerce (10), Social/Community (10), Productivity (10),
#             Education (10), Media (10), Communication (8),
#             Utility (10), Gaming/Fun (7)
# ═══════════════════════════════════════════════════════════════════════════════

# ─── 1. E-commerce patterns (10) ─────────────────────────────────────────────

ECOM_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state selected = "none"
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    grid columns: __COLS__, gap: 12px {
      each __IT__ in items | sort-by __SORT__ {
        rect padding: 16px, color: __BG__, radius: 8px {
          column gap: 4px {
            text "{__IT__.__F1__}" font-weight: bold
            text "{__IT__.__F2__}" color: __PRICE_CLR__
          }
          rect width: 80px, height: 32px, color: __BTN_CLR__, radius: 4px {
            text "__ACT__" color: #ffffff
            on click: set selected = "{__IT__.__F1__}"
          }
        }
      }
    }

    if selected != "none" {
      text "Selected: {selected}" color: #16a34a
    }
  }
}"""

for n, cfg in [
    ("product-listing", {"TITLE": "Product Catalog", "DESC": "E-commerce product listing with prices", "ITEMS": '{name: "Wireless Mouse", price: "$29"}, {name: "Keyboard", price: "$59"}, {name: "Monitor", price: "$349"}, {name: "Webcam", price: "$79"}', "LABEL": "products", "IT": "prod", "SORT": "name", "F1": "name", "F2": "price", "BG": "#f8fafc", "PRICE_CLR": "#2563eb", "BTN_CLR": "#2563eb", "ACT": "Add", "COLS": "2"}),
    ("wishlist", {"TITLE": "My Wishlist", "DESC": "Wishlist with saved items and prices", "ITEMS": '{name: "Headphones", price: "$199"}, {name: "Tablet", price: "$449"}, {name: "Speaker", price: "$89"}', "LABEL": "saved items", "IT": "wish", "SORT": "name", "F1": "name", "F2": "price", "BG": "#fdf2f8", "PRICE_CLR": "#ec4899", "BTN_CLR": "#ec4899", "ACT": "Buy", "COLS": "2"}),
    ("order-history", {"TITLE": "Order History", "DESC": "Past orders with status tracking", "ITEMS": '{name: "Laptop Stand", price: "Delivered"}, {name: "USB Hub", price: "Shipped"}, {name: "Cable Kit", price: "Processing"}', "LABEL": "orders", "IT": "order", "SORT": "name", "F1": "name", "F2": "price", "BG": "#ecfdf5", "PRICE_CLR": "#16a34a", "BTN_CLR": "#64748b", "ACT": "Track", "COLS": "1"}),
    ("compare", {"TITLE": "Compare Products", "DESC": "Side-by-side product comparison view", "ITEMS": '{name: "Phone A", price: "$799"}, {name: "Phone B", price: "$699"}, {name: "Phone C", price: "$599"}', "LABEL": "phones", "IT": "phone", "SORT": "name", "F1": "name", "F2": "price", "BG": "#eff6ff", "PRICE_CLR": "#1e40af", "BTN_CLR": "#3b82f6", "ACT": "Pick", "COLS": "3"}),
    ("category-browser", {"TITLE": "Categories", "DESC": "Product category browser with item counts", "ITEMS": '{name: "Electronics", price: "124 items"}, {name: "Clothing", price: "89 items"}, {name: "Home", price: "56 items"}, {name: "Sports", price: "42 items"}', "LABEL": "categories", "IT": "cat", "SORT": "name", "F1": "name", "F2": "price", "BG": "#fef3c7", "PRICE_CLR": "#92400e", "BTN_CLR": "#f59e0b", "ACT": "Browse", "COLS": "2"}),
    ("reviews", {"TITLE": "Customer Reviews", "DESC": "Product review display with star ratings", "ITEMS": '{name: "Great quality!", price: "5 stars"}, {name: "Good value", price: "4 stars"}, {name: "Decent product", price: "3 stars"}', "LABEL": "reviews", "IT": "rev", "SORT": "name", "F1": "name", "F2": "price", "BG": "#fefce8", "PRICE_CLR": "#f59e0b", "BTN_CLR": "#64748b", "ACT": "Helpful", "COLS": "1"}),
]:
    ex(f"gen-ecom-{n}.naze", cfg["DESC"], fill(ECOM_T, cfg))

ex("gen-ecom-cart.naze", "Shopping cart with quantity display and checkout",
   """-- Shopping cart
app "Shopping Cart" {
  state items = [{name: "Notebook", qty: "2", price: "12"}, {name: "Pen Set", qty: "1", price: "8"}, {name: "Eraser", qty: "3", price: "2"}]
  computed item-count = items | count

  column padding: 20px, gap: 16px {
    heading "Shopping Cart"
    text "{item-count} items in cart" color: #64748b

    each item in items {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 12px {
        column gap: 4px {
          text "{item.name}" font-weight: bold
          text "Qty: {item.qty}" color: #64748b
        }
        text "${item.price}" color: #2563eb, font-size: 18px
      }
    }

    separator
    rect width: 160px, height: 44px, color: #16a34a, radius: 8px {
      text "Proceed to Checkout" color: #ffffff
    }
  }
}""")

ex("gen-ecom-checkout.naze", "Multi-step checkout with shipping and payment",
   """-- Checkout flow
app "Checkout" {
  state name = ""
  state address = ""
  state card = ""
  state step = "shipping"

  column padding: 20px, gap: 16px {
    heading "Checkout"

    match step {
      "shipping": column gap: 12px {
        text "Shipping Info" font-weight: bold
        input bind: name, placeholder: "Full Name"
        input bind: address, placeholder: "Address"
        rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
          text "Next" color: #ffffff
          on click: set step = "payment"
        }
      }
      "payment": column gap: 12px {
        text "Payment Info" font-weight: bold
        input bind: card, placeholder: "Card Number"
        rect width: 120px, height: 40px, color: #16a34a, radius: 8px {
          text "Pay Now" color: #ffffff
          on click: set step = "done"
        }
      }
      "done": text "Order placed!" color: #16a34a, font-size: 24px
      _: text "Unknown step"
    }
  }
}""")

ex("gen-ecom-pricing.naze", "Pricing table with three plan tiers",
   """-- Pricing table
app "Pricing Plans" {
  state chosen = "none"

  column padding: 20px, gap: 16px {
    heading "Choose Your Plan"

    grid columns: 3, gap: 16px {
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Starter" font-weight: bold, font-size: 18px
          text "$9/mo" font-size: 28px, color: #64748b
          text "1 user" color: #94a3b8
          text "5GB storage" color: #94a3b8
          rect width: 100px, height: 36px, color: #64748b, radius: 8px {
            text "Select" color: #ffffff
            on click: set chosen = "starter"
          }
        }
      }
      rect padding: 20px, color: #eff6ff, radius: 12px {
        column gap: 8px {
          text "Pro" font-weight: bold, font-size: 18px
          text "$29/mo" font-size: 28px, color: #2563eb
          text "5 users" color: #64748b
          text "50GB storage" color: #64748b
          rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
            text "Select" color: #ffffff
            on click: set chosen = "pro"
          }
        }
      }
      rect padding: 20px, color: #f0fdf4, radius: 12px {
        column gap: 8px {
          text "Enterprise" font-weight: bold, font-size: 18px
          text "$99/mo" font-size: 28px, color: #16a34a
          text "Unlimited" color: #64748b
          text "500GB storage" color: #64748b
          rect width: 100px, height: 36px, color: #16a34a, radius: 8px {
            text "Select" color: #ffffff
            on click: set chosen = "enterprise"
          }
        }
      }
    }

    if chosen != "none" {
      text "Plan: {chosen}" color: #16a34a, font-size: 18px
    }
  }
}""")

ex("gen-ecom-inventory.naze", "Inventory dashboard with stock levels by SKU",
   """-- Inventory dashboard
app "Inventory Manager" {
  state items = [{name: "Blue Widget", sku: "BW-001", stock: "150"}, {name: "Red Gadget", sku: "RG-042", stock: "3"}, {name: "Green Tool", sku: "GT-017", stock: "45"}]
  computed total-skus = items | count

  column padding: 20px, gap: 16px {
    heading "Inventory Manager"
    text "{total-skus} SKUs tracked" color: #64748b

    each item in items | sort-by name {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        column gap: 2px {
          text "{item.name}" font-weight: bold
          text "SKU: {item.sku}" color: #94a3b8, font-size: 12px
        }
        text "Stock: {item.stock}" color: #2563eb
      }
    }
  }
}""")

# ─── 2. Social/Community patterns (10) ───────────────────────────────────────

SOCIAL_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state new-entry = ""
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    row gap: 8px {
      input bind: new-entry, placeholder: "__PLACEHOLDER__"
      rect width: 80px, height: 40px, color: __CLR__, radius: 8px {
        text "__BTN__" color: #ffffff
        on click: append new-entry to items
      }
    }

    each __IT__ in items {
      row padding: 12px, color: __BG__, radius: 8px, gap: 8px {
        text "{__IT__.__F1__}" font-weight: bold
        text "{__IT__.__F2__}" color: #64748b
      }
    }
  }
}"""

for n, cfg in [
    ("feed", {"TITLE": "Social Feed", "DESC": "Social media feed with posts and authors", "ITEMS": '{author: "Alice", body: "Just shipped a new feature!"}, {author: "Bob", body: "Great weather today"}, {author: "Carol", body: "Reading a new book"}', "LABEL": "posts", "IT": "post", "F1": "author", "F2": "body", "BG": "#f8fafc", "CLR": "#2563eb", "BTN": "Post", "PLACEHOLDER": "What is on your mind?"}),
    ("comments", {"TITLE": "Comments", "DESC": "Comment section with replies", "ITEMS": '{user: "Dave", text: "Great article!"}, {user: "Eve", text: "Very helpful, thanks"}, {user: "Frank", text: "I have a question..."}', "LABEL": "comments", "IT": "cmt", "F1": "user", "F2": "text", "BG": "#f0f9ff", "CLR": "#6366f1", "BTN": "Reply", "PLACEHOLDER": "Add a comment..."}),
    ("followers", {"TITLE": "Followers", "DESC": "Follower list with join dates", "ITEMS": '{name: "Grace", since: "Jan 2024"}, {name: "Hank", since: "Mar 2024"}, {name: "Ivy", since: "Jun 2024"}', "LABEL": "followers", "IT": "f", "F1": "name", "F2": "since", "BG": "#ecfdf5", "CLR": "#16a34a", "BTN": "Add", "PLACEHOLDER": "Search users..."}),
    ("activity", {"TITLE": "Activity Stream", "DESC": "Activity feed with user actions", "ITEMS": '{user: "Kim", action: "pushed to main"}, {user: "Leo", action: "opened PR #42"}, {user: "Mia", action: "closed issue #17"}', "LABEL": "events", "IT": "evt", "F1": "user", "F2": "action", "BG": "#faf5ff", "CLR": "#8b5cf6", "BTN": "Log", "PLACEHOLDER": "Log activity..."}),
    ("directory", {"TITLE": "User Directory", "DESC": "User directory with departments", "ITEMS": '{name: "Nina", dept: "Engineering"}, {name: "Oscar", dept: "Design"}, {name: "Pat", dept: "Marketing"}', "LABEL": "members", "IT": "member", "F1": "name", "F2": "dept", "BG": "#fff7ed", "CLR": "#f97316", "BTN": "Invite", "PLACEHOLDER": "Invite by email..."}),
    ("groups", {"TITLE": "Groups", "DESC": "Group listing with member counts", "ITEMS": '{name: "Rust Devs", members: "128 members"}, {name: "UI Design", members: "64 members"}, {name: "Open Source", members: "256 members"}', "LABEL": "groups", "IT": "grp", "F1": "name", "F2": "members", "BG": "#fefce8", "CLR": "#eab308", "BTN": "Create", "PLACEHOLDER": "New group name..."}),
]:
    ex(f"gen-social-{n}.naze", cfg["DESC"], fill(SOCIAL_T, cfg))

ex("gen-social-likes.naze", "Like and reaction counter with multiple types",
   """-- Reaction counter
app "Reactions" {
  state likes = 0
  state hearts = 0
  state laughs = 0
  computed total-reactions = likes + hearts + laughs

  column padding: 20px, gap: 16px {
    heading "Post Reactions"
    text "{total-reactions} total reactions" color: #64748b

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "This is a sample post that people can react to." font-size: 16px
    }

    row gap: 8px {
      rect width: 80px, height: 40px, color: #2563eb, radius: 8px {
        text "Like ({likes})" color: #ffffff
        on click: set likes = likes + 1
      }
      rect width: 80px, height: 40px, color: #ec4899, radius: 8px {
        text "Love ({hearts})" color: #ffffff
        on click: set hearts = hearts + 1
      }
      rect width: 80px, height: 40px, color: #f59e0b, radius: 8px {
        text "Haha ({laughs})" color: #ffffff
        on click: set laughs = laughs + 1
      }
    }
  }
}""")

ex("gen-social-notifications.naze", "Notification center with unread count",
   """-- Notification center
app "Notifications" {
  state notifs = [{title: "New follower", detail: "Alice followed you"}, {title: "Comment reply", detail: "Bob replied to your comment"}, {title: "Like", detail: "Carol liked your post"}]
  state unread = 3
  computed notif-count = notifs | count

  column padding: 20px, gap: 16px {
    heading "Notifications"
    text "{unread} unread of {notif-count}" color: #dc2626

    each n in notifs {
      row padding: 12px, color: #fef2f2, radius: 8px, gap: 8px {
        column gap: 2px {
          text "{n.title}" font-weight: bold
          text "{n.detail}" color: #64748b, font-size: 14px
        }
      }
    }

    rect width: 120px, height: 36px, color: #2563eb, radius: 8px {
      text "Mark all read" color: #ffffff
      on click: set unread = 0
    }
  }
}""")

ex("gen-social-reactions.naze", "Emoji reaction picker for messages",
   """-- Emoji reaction picker
app "React to Message" {
  state reaction = "none"

  column padding: 20px, gap: 16px {
    heading "Message"
    rect padding: 16px, color: #f0f9ff, radius: 8px {
      text "Hey team, the release looks great!" font-size: 16px
    }

    text "Your reaction: {reaction}" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 40px, color: #fef3c7, radius: 8px {
        text "Thumbs"
        on click: set reaction = "thumbs-up"
      }
      rect width: 60px, height: 40px, color: #fce7f3, radius: 8px {
        text "Heart"
        on click: set reaction = "heart"
      }
      rect width: 60px, height: 40px, color: #ecfdf5, radius: 8px {
        text "Check"
        on click: set reaction = "check"
      }
      rect width: 60px, height: 40px, color: #eff6ff, radius: 8px {
        text "Eyes"
        on click: set reaction = "eyes"
      }
    }
  }
}""")

ex("gen-social-mentions.naze", "Mention tracker showing who mentioned you",
   """-- Mention tracker
app "Mentions" {
  state mentions = [{from: "Alice", context: "in #general"}, {from: "Bob", context: "in PR review"}, {from: "Carol", context: "in thread"}]
  computed mention-count = mentions | count

  column padding: 20px, gap: 16px {
    heading "Your Mentions"
    text "{mention-count} recent mentions" color: #6366f1

    each m in mentions {
      row padding: 10px, color: #faf5ff, radius: 6px, gap: 8px {
        text "@{m.from}" font-weight: bold, color: #7c3aed
        text "{m.context}" color: #64748b
      }
    }
  }
}""")

# ─── 3. Productivity patterns (10) ───────────────────────────────────────────

PROD_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state view = "__DEFAULT_VIEW__"
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 32px, color: __C1__, radius: 4px {
        text "__V1__" color: #ffffff
        on click: set view = "__V1_KEY__"
      }
      rect width: 80px, height: 32px, color: __C2__, radius: 4px {
        text "__V2__" color: #ffffff
        on click: set view = "__V2_KEY__"
      }
    }

    each __IT__ in items | sort-by __SORT__ {
      row padding: 12px, color: __BG__, radius: 4px, gap: 12px {
        text "{__IT__.__F1__}" font-weight: bold
        text "{__IT__.__F2__}" color: __F2_CLR__
      }
    }
  }
}"""

for n, cfg in [
    ("task-board", {"TITLE": "Task Board", "DESC": "Task board with status filtering", "ITEMS": '{title: "Fix login bug", status: "in-progress"}, {title: "Write tests", status: "todo"}, {title: "Deploy v2", status: "done"}, {title: "Update docs", status: "todo"}', "LABEL": "tasks", "IT": "task", "SORT": "status", "F1": "title", "F2": "status", "BG": "#f8fafc", "C1": "#2563eb", "C2": "#64748b", "V1": "Board", "V1_KEY": "board", "V2": "List", "V2_KEY": "list", "DEFAULT_VIEW": "board", "F2_CLR": "#6366f1"}),
    ("time-tracker", {"TITLE": "Time Tracker", "DESC": "Time entries with project and hours", "ITEMS": '{project: "Website", hours: "3.5h"}, {project: "API", hours: "2h"}, {project: "Design", hours: "1.5h"}, {project: "Meetings", hours: "1h"}', "LABEL": "entries", "IT": "entry", "SORT": "project", "F1": "project", "F2": "hours", "BG": "#ecfdf5", "C1": "#16a34a", "C2": "#64748b", "V1": "Today", "V1_KEY": "today", "V2": "Week", "V2_KEY": "week", "DEFAULT_VIEW": "today", "F2_CLR": "#16a34a"}),
    ("project-list", {"TITLE": "Projects", "DESC": "Project list with deadlines", "ITEMS": '{name: "Alpha", deadline: "Mar 15"}, {name: "Beta", deadline: "Apr 1"}, {name: "Gamma", deadline: "May 10"}', "LABEL": "projects", "IT": "proj", "SORT": "name", "F1": "name", "F2": "deadline", "BG": "#eff6ff", "C1": "#3b82f6", "C2": "#94a3b8", "V1": "Active", "V1_KEY": "active", "V2": "Archive", "V2_KEY": "archive", "DEFAULT_VIEW": "active", "F2_CLR": "#dc2626"}),
    ("weekly-planner", {"TITLE": "Weekly Planner", "DESC": "Weekly planner with daily tasks", "ITEMS": '{day: "Monday", task: "Sprint planning"}, {day: "Tuesday", task: "Code review"}, {day: "Wednesday", task: "Feature dev"}, {day: "Thursday", task: "Testing"}, {day: "Friday", task: "Retrospective"}', "LABEL": "activities", "IT": "act", "SORT": "day", "F1": "day", "F2": "task", "BG": "#fef3c7", "C1": "#f59e0b", "C2": "#64748b", "V1": "This Week", "V1_KEY": "current", "V2": "Next", "V2_KEY": "next", "DEFAULT_VIEW": "current", "F2_CLR": "#92400e"}),
    ("milestones", {"TITLE": "Milestones", "DESC": "Milestone tracker with completion status", "ITEMS": '{name: "MVP Launch", status: "done"}, {name: "Beta Release", status: "in-progress"}, {name: "Public Launch", status: "planned"}', "LABEL": "milestones", "IT": "ms", "SORT": "name", "F1": "name", "F2": "status", "BG": "#fdf2f8", "C1": "#ec4899", "C2": "#64748b", "V1": "Timeline", "V1_KEY": "timeline", "V2": "Grid", "V2_KEY": "grid", "DEFAULT_VIEW": "timeline", "F2_CLR": "#be185d"}),
]:
    ex(f"gen-prod-{n}.naze", cfg["DESC"], fill(PROD_T, cfg))

ex("gen-prod-note-editor.naze", "Note editor with title body and save",
   """-- Note editor
app "Note Editor" {
  state title = ""
  state body = ""
  state saved = false

  column padding: 20px, gap: 16px {
    heading "Note Editor"
    input bind: title, placeholder: "Note title..."
    textarea bind: body, placeholder: "Start writing..."

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 8px {
        text "Save" color: #ffffff
        on click: set saved = true
      }
      rect width: 80px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Clear"
        on click: set title = ""
      }
    }

    if saved {
      text "Saved!" color: #16a34a
    }
  }
}""")

ex("gen-prod-goal-tracker.naze", "Goal tracker with progress toward targets",
   """-- Goal tracker
app "Goal Tracker" {
  state goals = [{name: "Read 12 books", done: "8", target: "12"}, {name: "Run 100km", done: "67", target: "100"}, {name: "Save $5000", done: "3200", target: "5000"}]
  computed goal-count = goals | count

  column padding: 20px, gap: 16px {
    heading "My Goals"
    text "{goal-count} active goals" color: #64748b

    each g in goals {
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        column gap: 4px {
          text "{g.name}" font-weight: bold
          text "{g.done} / {g.target}" color: #16a34a, font-size: 18px
        }
      }
    }
  }
}""")

ex("gen-prod-priority-matrix.naze", "Eisenhower priority matrix with four quadrants",
   """-- Priority matrix
app "Priority Matrix" {
  state urgent-important = ["Fix production bug", "Client deadline"]
  state not-urgent-important = ["Strategic planning", "Team training"]

  column padding: 20px, gap: 16px {
    heading "Priority Matrix"

    grid columns: 2, gap: 12px {
      rect padding: 12px, color: #fef2f2, radius: 8px {
        column gap: 4px {
          text "Do First" font-weight: bold, color: #dc2626
          each item in urgent-important {
            text "{item}" font-size: 14px
          }
        }
      }
      rect padding: 12px, color: #eff6ff, radius: 8px {
        column gap: 4px {
          text "Schedule" font-weight: bold, color: #2563eb
          each item in not-urgent-important {
            text "{item}" font-size: 14px
          }
        }
      }
    }
  }
}""")

ex("gen-prod-habit-tracker.naze", "Daily habit tracker with streak counters",
   """-- Daily habit tracker
app "Habit Tracker" {
  state habits = [{name: "Morning run", streak: "12"}, {name: "Read 30min", streak: "8"}, {name: "No sugar", streak: "5"}, {name: "Journal", streak: "21"}]
  state day = 1

  timer next-day: every 1s {
    set day = day + 1
  }

  column padding: 20px, gap: 16px {
    heading "Daily Habits"
    text "Day {day}" color: #64748b

    each h in habits | sort-by streak {
      row padding: 12px, color: #ecfdf5, radius: 8px, gap: 12px {
        text "{h.name}" font-weight: bold
        text "{h.streak} day streak" color: #16a34a
      }
    }
  }
}""")

ex("gen-prod-calendar.naze", "Calendar with weekly events and day selection",
   """-- Calendar events
app "Calendar" {
  state events = [{day: "Mon", event: "Team standup"}, {day: "Tue", event: "Design review"}, {day: "Wed", event: "Sprint demo"}, {day: "Thu", event: "1-on-1s"}, {day: "Fri", event: "Retrospective"}]
  state selected-day = "Mon"

  column padding: 20px, gap: 16px {
    heading "This Week"

    row gap: 4px {
      each evt in events {
        rect width: 60px, height: 40px, color: #e2e8f0, radius: 4px {
          text "{evt.day}"
          on click: set selected-day = "{evt.day}"
        }
      }
    }

    text "Selected: {selected-day}" color: #2563eb, font-size: 18px

    each evt in events {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{evt.day}" font-weight: bold, color: #64748b
        text "{evt.event}"
      }
    }
  }
}""")

# ─── 4. Education patterns (10) ──────────────────────────────────────────────

EDU_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state current = 0
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{current} of {total} __LABEL__" color: __CLR__

    each __IT__ in items {
      rect padding: 12px, color: __BG__, radius: 8px {
        column gap: 4px {
          text "{__IT__.__F1__}" font-weight: bold
          text "{__IT__.__F2__}" color: #64748b
        }
      }
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: __CLR__, radius: 8px {
        text "__ACT__" color: #ffffff
        on click: set current = current + 1
      }
      rect width: 100px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Reset"
        on click: set current = 0
      }
    }
  }
}"""

for n, cfg in [
    ("quiz", {"TITLE": "Quick Quiz", "DESC": "Quiz app with questions and score tracking", "ITEMS": '{question: "What is 2+2?", answer: "4"}, {question: "Capital of Japan?", answer: "Tokyo"}, {question: "Largest planet?", answer: "Jupiter"}', "LABEL": "answered", "IT": "q", "F1": "question", "F2": "answer", "BG": "#eff6ff", "CLR": "#2563eb", "ACT": "Next"}),
    ("flashcards", {"TITLE": "Study Flashcards", "DESC": "Flashcard deck with terms and definitions", "ITEMS": '{term: "Algorithm", def: "Step-by-step procedure"}, {term: "Variable", def: "Named storage location"}, {term: "Function", def: "Reusable code block"}', "LABEL": "reviewed", "IT": "card", "F1": "term", "F2": "def", "BG": "#fef3c7", "CLR": "#f59e0b", "ACT": "Flip"}),
    ("course-list", {"TITLE": "My Courses", "DESC": "Course listing with instructors", "ITEMS": '{title: "Intro to Rust", instructor: "Dr. Smith"}, {title: "Web Design", instructor: "Prof. Lee"}, {title: "Data Science", instructor: "Dr. Chen"}', "LABEL": "completed", "IT": "course", "F1": "title", "F2": "instructor", "BG": "#ecfdf5", "CLR": "#16a34a", "ACT": "Continue"}),
    ("grade-book", {"TITLE": "Grade Book", "DESC": "Grade book with subjects and scores", "ITEMS": '{subject: "Math", grade: "A (95%)"}, {subject: "Science", grade: "B+ (88%)"}, {subject: "English", grade: "A- (91%)"}, {subject: "History", grade: "B (85%)"}', "LABEL": "graded", "IT": "entry", "F1": "subject", "F2": "grade", "BG": "#fdf2f8", "CLR": "#ec4899", "ACT": "View"}),
    ("vocabulary", {"TITLE": "Vocabulary", "DESC": "Vocabulary builder with definitions", "ITEMS": '{word: "Ephemeral", meaning: "Lasting a short time"}, {word: "Ubiquitous", meaning: "Found everywhere"}, {word: "Pragmatic", meaning: "Practical approach"}', "LABEL": "learned", "IT": "v", "F1": "word", "F2": "meaning", "BG": "#faf5ff", "CLR": "#8b5cf6", "ACT": "Learn"}),
    ("reading-list", {"TITLE": "Reading List", "DESC": "Reading list with books and authors", "ITEMS": '{title: "Clean Code", author: "Robert Martin"}, {title: "Designing Data Apps", author: "Martin Kleppmann"}, {title: "The Pragmatic Programmer", author: "Hunt and Thomas"}', "LABEL": "read", "IT": "book", "F1": "title", "F2": "author", "BG": "#fff7ed", "CLR": "#f97316", "ACT": "Mark Read"}),
    ("progress", {"TITLE": "Learning Progress", "DESC": "Learning progress with module status", "ITEMS": '{module: "Basics", status: "Complete"}, {module: "Intermediate", status: "In Progress"}, {module: "Advanced", status: "Locked"}', "LABEL": "done", "IT": "mod", "F1": "module", "F2": "status", "BG": "#f0f9ff", "CLR": "#0ea5e9", "ACT": "Start Next"}),
]:
    ex(f"gen-edu-{n}.naze", cfg["DESC"], fill(EDU_T, cfg))

ex("gen-edu-study-timer.naze", "Pomodoro study timer with session counter",
   """-- Study timer
app "Study Timer" {
  state minutes = 25
  state sessions = 0
  state mode = "focus"

  timer countdown: every 1s {
    set minutes = minutes - 1
  }

  column padding: 20px, gap: 16px {
    heading "Study Timer"
    text "{minutes} min remaining" font-size: 36px, color: #dc2626

    match mode {
      "focus": text "Focus Session" color: #dc2626, font-size: 18px
      "break": text "Take a Break" color: #16a34a, font-size: 18px
      _: text "Ready"
    }

    text "Sessions: {sessions}" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "Focus" color: #ffffff
        on click: set mode = "focus"
      }
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Break" color: #ffffff
        on click: set mode = "break"
      }
      rect width: 80px, height: 40px, color: #64748b, radius: 8px {
        text "Reset" color: #ffffff
        on click: set minutes = 25
      }
    }
  }
}""")

ex("gen-edu-exercise.naze", "Exercise tracker with sets and reps",
   """-- Exercise tracker
app "Exercise Log" {
  state exercises = [{name: "Push-ups", sets: "3", reps: "15"}, {name: "Squats", sets: "4", reps: "12"}, {name: "Planks", sets: "3", reps: "60s"}]
  computed exercise-count = exercises | count

  column padding: 20px, gap: 16px {
    heading "Today's Workout"
    text "{exercise-count} exercises" color: #64748b

    each ex in exercises {
      row padding: 12px, color: #ecfdf5, radius: 8px, gap: 12px {
        text "{ex.name}" font-weight: bold
        text "{ex.sets} x {ex.reps}" color: #16a34a
      }
    }
  }
}""")

ex("gen-edu-skill-tree.naze", "Skill tree with prerequisites and unlock status",
   """-- Skill tree
app "Skill Tree" {
  state skills = [{name: "HTML Basics", status: "unlocked"}, {name: "CSS Layout", status: "unlocked"}, {name: "JavaScript", status: "locked"}, {name: "React", status: "locked"}]
  computed unlocked = 2

  column padding: 20px, gap: 16px {
    heading "Skill Tree"
    text "{unlocked} skills unlocked" color: #6366f1

    each skill in skills {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 8px {
        text "{skill.name}" font-weight: bold
        match skill.status {
          "unlocked": text "Available" color: #16a34a
          "locked": text "Locked" color: #dc2626
          _: text "Unknown" color: #64748b
        }
      }
    }
  }
}""")

# ─── 5. Media patterns (10) ──────────────────────────────────────────────────

MEDIA_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state now-playing = "none"
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    if now-playing != "none" {
      rect padding: 12px, color: __ACCENT__, radius: 8px {
        text "Now: {now-playing}" color: #ffffff, font-size: 18px
      }
    }

    each __IT__ in items {
      row padding: 12px, color: __BG__, radius: 8px, gap: 12px {
        text "{__IT__.__F1__}" font-weight: bold
        text "{__IT__.__F2__}" color: #64748b
        rect width: 60px, height: 28px, color: __CLR__, radius: 4px {
          text "__ACT__" color: #ffffff
          on click: set now-playing = "{__IT__.__F1__}"
        }
      }
    }
  }
}"""

for n, cfg in [
    ("gallery", {"TITLE": "Photo Gallery", "DESC": "Image gallery with titles and dates", "ITEMS": '{title: "Sunset Beach", date: "Jan 5"}, {title: "Mountain View", date: "Jan 12"}, {title: "City Lights", date: "Jan 20"}, {title: "Forest Trail", date: "Feb 3"}', "LABEL": "photos", "IT": "photo", "F1": "title", "F2": "date", "BG": "#fdf2f8", "CLR": "#ec4899", "ACCENT": "#be185d", "ACT": "View"}),
    ("podcast-list", {"TITLE": "Podcasts", "DESC": "Podcast listing with durations", "ITEMS": '{title: "Tech Talk #42", duration: "45 min"}, {title: "Design Hour", duration: "32 min"}, {title: "Startup Stories", duration: "58 min"}', "LABEL": "episodes", "IT": "ep", "F1": "title", "F2": "duration", "BG": "#faf5ff", "CLR": "#8b5cf6", "ACCENT": "#6d28d9", "ACT": "Play"}),
    ("music-queue", {"TITLE": "Music Queue", "DESC": "Music queue with tracks and artists", "ITEMS": '{title: "Bohemian Rhapsody", artist: "Queen"}, {title: "Imagine", artist: "Lennon"}, {title: "Billie Jean", artist: "MJ"}, {title: "Stairway", artist: "Led Zeppelin"}', "LABEL": "tracks", "IT": "track", "F1": "title", "F2": "artist", "BG": "#ecfdf5", "CLR": "#16a34a", "ACCENT": "#15803d", "ACT": "Play"}),
    ("media-library", {"TITLE": "Media Library", "DESC": "Media library with file types", "ITEMS": '{name: "vacation.mp4", type: "Video"}, {name: "song.mp3", type: "Audio"}, {name: "photo.jpg", type: "Image"}, {name: "doc.pdf", type: "Document"}', "LABEL": "files", "IT": "file", "F1": "name", "F2": "type", "BG": "#eff6ff", "CLR": "#2563eb", "ACCENT": "#1d4ed8", "ACT": "Open"}),
    ("playlist-mgr", {"TITLE": "Playlist Manager", "DESC": "Playlist manager with durations", "ITEMS": '{song: "Yesterday", length: "2:05"}, {song: "Hey Jude", length: "7:11"}, {song: "Let It Be", length: "3:50"}', "LABEL": "songs", "IT": "s", "F1": "song", "F2": "length", "BG": "#fefce8", "CLR": "#eab308", "ACCENT": "#a16207", "ACT": "Play"}),
]:
    ex(f"gen-media-{n}.naze", cfg["DESC"], fill(MEDIA_T, cfg))

ex("gen-media-video-player.naze", "Video player with playback controls",
   """-- Video player
app "Video Player" {
  state playing = false
  state position = 0
  state title = "Introduction to Naze"

  timer playback: every 1s {
    set position = position + 1
  }

  column padding: 20px, gap: 16px {
    heading "Video Player"

    rect width: 400px, height: 225px, color: #1e293b, radius: 8px {
      text "{title}" color: #ffffff, font-size: 18px
    }

    text "{position}s elapsed" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Play" color: #ffffff
        on click: set playing = true
      }
      rect width: 80px, height: 40px, color: #64748b, radius: 8px {
        text "Pause" color: #ffffff
        on click: set playing = false
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "Restart" color: #ffffff
        on click: set position = 0
      }
    }
  }
}""")

ex("gen-media-carousel.naze", "Image carousel with prev and next navigation",
   """-- Image carousel
app "Carousel" {
  state slide = 0

  column padding: 20px, gap: 16px {
    heading "Photo Carousel"

    rect width: 400px, height: 250px, color: #1e293b, radius: 12px, padding: 20px {
      text "Slide {slide}" color: #ffffff, font-size: 24px
    }

    row gap: 8px {
      rect width: 100px, height: 40px, color: #64748b, radius: 8px {
        text "Previous" color: #ffffff
        on click: set slide = slide - 1
      }
      rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
        text "Next" color: #ffffff
        on click: set slide = slide + 1
      }
    }
  }
}""")

ex("gen-media-thumbnail-grid.naze", "Thumbnail grid with selection state",
   """-- Thumbnail grid
app "Thumbnails" {
  state selected = "none"

  column padding: 20px, gap: 16px {
    heading "Photo Grid"
    text "Selected: {selected}" color: #64748b

    grid columns: 3, gap: 8px {
      rect width: 120px, height: 90px, color: #fecaca, radius: 8px {
        text "Photo 1"
        on click: set selected = "photo-1"
      }
      rect width: 120px, height: 90px, color: #bfdbfe, radius: 8px {
        text "Photo 2"
        on click: set selected = "photo-2"
      }
      rect width: 120px, height: 90px, color: #bbf7d0, radius: 8px {
        text "Photo 3"
        on click: set selected = "photo-3"
      }
      rect width: 120px, height: 90px, color: #fef08a, radius: 8px {
        text "Photo 4"
        on click: set selected = "photo-4"
      }
      rect width: 120px, height: 90px, color: #e9d5ff, radius: 8px {
        text "Photo 5"
        on click: set selected = "photo-5"
      }
      rect width: 120px, height: 90px, color: #fecdd3, radius: 8px {
        text "Photo 6"
        on click: set selected = "photo-6"
      }
    }
  }
}""")

ex("gen-media-audio-mixer.naze", "Audio mixer with channel volume controls",
   """-- Audio mixer
app "Audio Mixer" {
  state master = 80
  state vocals = 70
  state drums = 90
  state bass = 60

  column padding: 20px, gap: 16px {
    heading "Audio Mixer"

    column gap: 12px {
      row padding: 8px, gap: 12px {
        text "Master: {master}%" font-weight: bold, color: #2563eb
        rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
          text "+" color: #ffffff
          on click: set master = master + 5
        }
        rect width: 40px, height: 28px, color: #64748b, radius: 4px {
          text "-" color: #ffffff
          on click: set master = master - 5
        }
      }
      row padding: 8px, gap: 12px {
        text "Vocals: {vocals}%" font-weight: bold, color: #ec4899
        rect width: 40px, height: 28px, color: #ec4899, radius: 4px {
          text "+" color: #ffffff
          on click: set vocals = vocals + 5
        }
        rect width: 40px, height: 28px, color: #64748b, radius: 4px {
          text "-" color: #ffffff
          on click: set vocals = vocals - 5
        }
      }
      row padding: 8px, gap: 12px {
        text "Drums: {drums}%" font-weight: bold, color: #f59e0b
        rect width: 40px, height: 28px, color: #f59e0b, radius: 4px {
          text "+" color: #ffffff
          on click: set drums = drums + 5
        }
        rect width: 40px, height: 28px, color: #64748b, radius: 4px {
          text "-" color: #ffffff
          on click: set drums = drums - 5
        }
      }
    }
  }
}""")

ex("gen-media-slideshow.naze", "Auto-advancing slideshow with timer",
   """-- Auto slideshow
app "Slideshow" {
  state slide-num = 0

  timer advance: every 3s {
    set slide-num = slide-num + 1
  }

  column padding: 20px, gap: 16px {
    heading "Slideshow"

    rect width: 400px, height: 250px, color: #1e293b, radius: 12px, padding: 20px {
      text "Slide {slide-num}" color: #ffffff, font-size: 32px
    }

    text "Auto-advances every 3 seconds" color: #94a3b8

    rect width: 100px, height: 36px, color: #dc2626, radius: 8px {
      text "Restart" color: #ffffff
      on click: set slide-num = 0
    }
  }
}""")

# ─── 6. Communication patterns (8) ───────────────────────────────────────────

COMM_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  state __INPUT_NAME__ = ""
  state __STATUS__ = "__STATUS_INIT__"
  computed total = items | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total} __LABEL__" color: #64748b

    __INPUT_EL__

    rect width: __BW__px, height: 40px, color: __CLR__, radius: 8px {
      text "__BTN__" color: #ffffff
      on click: set __STATUS__ = "__STATUS_DONE__"
    }

    if __STATUS__ == "__STATUS_DONE__" {
      text "__SUCCESS_MSG__" color: #16a34a
    }

    each __IT__ in items {
      row padding: 12px, color: __BG__, radius: 4px, gap: 8px {
        text "{__IT__.__F1__}" font-weight: bold
        text "{__IT__.__F2__}" color: #64748b
      }
    }
  }
}"""

for n, cfg in [
    ("inbox", {"TITLE": "Email Inbox", "DESC": "Email inbox with sender and subject", "ITEMS": '{from: "Alice", subject: "Meeting at 3pm"}, {from: "Bob", subject: "Code review needed"}, {from: "Carol", subject: "Lunch tomorrow?"}', "LABEL": "emails", "IT": "email", "F1": "from", "F2": "subject", "BG": "#f8fafc", "CLR": "#2563eb", "BTN": "Compose", "BW": "120", "INPUT_NAME": "draft", "INPUT_EL": 'input bind: draft, placeholder: "New email..."', "STATUS": "sent", "STATUS_INIT": "idle", "STATUS_DONE": "sent", "SUCCESS_MSG": "Email sent!"}),
    ("thread", {"TITLE": "Message Thread", "DESC": "Message thread with replies", "ITEMS": '{sender: "Dave", msg: "Has anyone seen the report?"}, {sender: "Eve", msg: "I uploaded it yesterday"}, {sender: "Frank", msg: "Thanks, found it!"}', "LABEL": "messages", "IT": "m", "F1": "sender", "F2": "msg", "BG": "#f0f9ff", "CLR": "#6366f1", "BTN": "Reply", "BW": "100", "INPUT_NAME": "reply", "INPUT_EL": 'input bind: reply, placeholder: "Type reply..."', "STATUS": "replied", "STATUS_INIT": "idle", "STATUS_DONE": "replied", "SUCCESS_MSG": "Reply sent!"}),
    ("announcement", {"TITLE": "Announcements", "DESC": "Announcement board with dates", "ITEMS": '{title: "Office Closure", date: "Feb 14"}, {title: "New Policy", date: "Feb 10"}, {title: "Team Outing", date: "Feb 20"}', "LABEL": "announcements", "IT": "ann", "F1": "title", "F2": "date", "BG": "#fef3c7", "CLR": "#f59e0b", "BTN": "Post New", "BW": "120", "INPUT_NAME": "new-post", "INPUT_EL": 'input bind: new-post, placeholder: "New announcement..."', "STATUS": "posted", "STATUS_INIT": "idle", "STATUS_DONE": "posted", "SUCCESS_MSG": "Posted!"}),
    ("tickets", {"TITLE": "Support Tickets", "DESC": "Support ticket list with status", "ITEMS": '{title: "Login issue", status: "Open"}, {title: "Payment failed", status: "In Progress"}, {title: "Feature request", status: "Closed"}', "LABEL": "tickets", "IT": "ticket", "F1": "title", "F2": "status", "BG": "#fdf2f8", "CLR": "#ec4899", "BTN": "New Ticket", "BW": "120", "INPUT_NAME": "issue", "INPUT_EL": 'input bind: issue, placeholder: "Describe issue..."', "STATUS": "submitted", "STATUS_INIT": "idle", "STATUS_DONE": "submitted", "SUCCESS_MSG": "Ticket submitted!"}),
]:
    ex(f"gen-comm-{n}.naze", cfg["DESC"], fill(COMM_T, cfg))

ex("gen-comm-chat-room.naze", "Chat room with message list and send",
   """-- Chat room
app "Chat Room" {
  state message = ""
  state messages = [{user: "Alice", text: "Hey everyone!"}, {user: "Bob", text: "Hi Alice!"}, {user: "Carol", text: "What are we working on?"}]
  computed msg-count = messages | count

  column padding: 20px, gap: 16px {
    heading "Chat Room"
    text "{msg-count} messages" color: #64748b

    scroll height: 250px {
      column gap: 4px {
        each msg in messages {
          row padding: 8px, color: #f3f4f6, radius: 8px, gap: 8px {
            text "{msg.user}:" font-weight: bold, color: #2563eb
            text "{msg.text}"
          }
        }
      }
    }

    row gap: 8px {
      input bind: message, placeholder: "Type a message..."
      rect width: 80px, height: 40px, color: #2563eb, radius: 8px {
        text "Send" color: #ffffff
        on click: append message to messages
      }
    }
  }
}""")

ex("gen-comm-contact-form.naze", "Contact form with multiple fields",
   """-- Contact form
app "Contact Us" {
  state name = ""
  state email = ""
  state message = ""
  state submitted = false

  column padding: 20px, gap: 16px {
    heading "Contact Us"
    input bind: name, placeholder: "Your Name"
    input bind: email, placeholder: "Email Address"
    textarea bind: message, placeholder: "Your message..."

    rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
      text "Send Message" color: #ffffff
      on click: set submitted = true
    }

    if submitted {
      text "Thank you, {name}!" color: #16a34a
    }
  }
}""")

ex("gen-comm-feedback.naze", "Feedback form with star rating",
   """-- Feedback form
app "Give Feedback" {
  state rating = 0
  state comment = ""
  state submitted = false

  column padding: 20px, gap: 16px {
    heading "How was your experience?"
    text "Rating: {rating} / 5" font-size: 18px, color: #f59e0b

    row gap: 4px {
      rect width: 50px, height: 40px, color: #fef3c7, radius: 8px {
        text "1"
        on click: set rating = 1
      }
      rect width: 50px, height: 40px, color: #fef3c7, radius: 8px {
        text "2"
        on click: set rating = 2
      }
      rect width: 50px, height: 40px, color: #fef3c7, radius: 8px {
        text "3"
        on click: set rating = 3
      }
      rect width: 50px, height: 40px, color: #fef3c7, radius: 8px {
        text "4"
        on click: set rating = 4
      }
      rect width: 50px, height: 40px, color: #fef3c7, radius: 8px {
        text "5"
        on click: set rating = 5
      }
    }

    textarea bind: comment, placeholder: "Additional comments..."

    rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
      text "Submit" color: #ffffff
      on click: set submitted = true
    }

    if submitted {
      text "Thanks for your feedback!" color: #16a34a
    }
  }
}""")

ex("gen-comm-comment-section.naze", "Comment section with add functionality",
   """-- Comment section
app "Comments" {
  state new-comment = ""
  state comments = [{author: "Guest1", body: "This is fantastic!"}, {author: "Guest2", body: "I learned a lot"}, {author: "Guest3", body: "Cover more topics?"}]
  computed comment-total = comments | count

  column padding: 20px, gap: 16px {
    heading "Comments"
    text "{comment-total} comments" color: #64748b

    each c in comments {
      column padding: 12px, color: #f8fafc, radius: 8px, gap: 4px {
        text "{c.author}" font-weight: bold, color: #6366f1
        text "{c.body}"
      }
    }

    separator

    row gap: 8px {
      input bind: new-comment, placeholder: "Add a comment..."
      rect width: 100px, height: 40px, color: #6366f1, radius: 8px {
        text "Comment" color: #ffffff
        on click: append new-comment to comments
      }
    }
  }
}""")

# ─── 7. Utility patterns (10) ────────────────────────────────────────────────

ex("gen-util-color-picker.naze", "Color picker with RGB value controls",
   """-- Color picker
app "Color Picker" {
  state r = 37
  state g = 99
  state b = 235

  column padding: 20px, gap: 16px {
    heading "Color Picker"

    rect width: 200px, height: 100px, color: #2563eb, radius: 12px

    row gap: 8px {
      text "R: {r}" color: #ef4444
      rect width: 40px, height: 28px, color: #ef4444, radius: 4px {
        text "+" color: #ffffff
        on click: set r = r + 10
      }
    }
    row gap: 8px {
      text "G: {g}" color: #16a34a
      rect width: 40px, height: 28px, color: #16a34a, radius: 4px {
        text "+" color: #ffffff
        on click: set g = g + 10
      }
    }
    row gap: 8px {
      text "B: {b}" color: #2563eb
      rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
        text "+" color: #ffffff
        on click: set b = b + 10
      }
    }
  }
}""")

ex("gen-util-unit-converter.naze", "Metric to imperial unit converter",
   """-- Unit converter
app "Unit Converter" {
  state value = 100
  state unit = "km"
  computed converted = value * 62 / 100

  column padding: 20px, gap: 16px {
    heading "Unit Converter"
    text "{value} {unit}" font-size: 24px
    text "= {converted} miles" font-size: 24px, color: #2563eb

    row gap: 8px {
      rect width: 60px, height: 36px, color: #2563eb, radius: 4px {
        text "+10" color: #ffffff
        on click: set value = value + 10
      }
      rect width: 60px, height: 36px, color: #64748b, radius: 4px {
        text "-10" color: #ffffff
        on click: set value = value - 10
      }
      rect width: 80px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Reset"
        on click: set value = 100
      }
    }
  }
}""")

ex("gen-util-countdown.naze", "Countdown timer to a specific event",
   """-- Event countdown
app "Event Countdown" {
  state days = 30

  timer tick: every 1s {
    set days = days - 1
  }

  column padding: 20px, gap: 16px {
    heading "Launch Day"
    text "{days}" font-size: 72px, color: #dc2626
    text "days remaining" color: #64748b, font-size: 18px

    rect width: 100px, height: 36px, color: #64748b, radius: 8px {
      text "Reset" color: #ffffff
      on click: set days = 30
    }
  }
}""")

ex("gen-util-stopwatch.naze", "Stopwatch with lap recording",
   """-- Stopwatch
app "Stopwatch" {
  state seconds = 0
  state laps = []

  timer clock: every 1s {
    set seconds = seconds + 1
  }

  column padding: 20px, gap: 16px {
    heading "Stopwatch"
    text "{seconds}s" font-size: 48px, color: #2563eb

    row gap: 8px {
      rect width: 80px, height: 40px, color: #f59e0b, radius: 8px {
        text "Lap" color: #ffffff
        on click: append seconds to laps
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset" color: #ffffff
        on click: set seconds = 0
      }
    }

    each lap in laps {
      text "Lap: {lap}s" color: #64748b
    }
  }
}""")

ex("gen-util-random-gen.naze", "Random number display with roll counter",
   """-- Random display
app "Random Generator" {
  state value = 42
  state rolls = 0
  computed doubled = value * 2

  column padding: 20px, gap: 16px {
    heading "Random Generator"
    text "{value}" font-size: 64px, color: #6366f1
    text "Doubled: {doubled}" color: #64748b
    text "Rolls: {rolls}" color: #94a3b8

    row gap: 8px {
      rect width: 80px, height: 40px, color: #6366f1, radius: 8px {
        text "Roll" color: #ffffff
        on click: set rolls = rolls + 1
      }
      rect width: 80px, height: 40px, color: #e2e8f0, radius: 4px {
        text "Reset"
        on click: set value = 42
      }
    }
  }
}""")

ex("gen-util-markdown-preview.naze", "Markdown editor with edit and preview toggle",
   """-- Markdown editor
app "Markdown Editor" {
  state content = ""
  state mode = "edit"

  column padding: 20px, gap: 16px {
    heading "Markdown Editor"

    row gap: 8px {
      rect width: 80px, height: 32px, color: #2563eb, radius: 4px {
        text "Edit" color: #ffffff
        on click: set mode = "edit"
      }
      rect width: 80px, height: 32px, color: #16a34a, radius: 4px {
        text "Preview" color: #ffffff
        on click: set mode = "preview"
      }
    }

    match mode {
      "edit": textarea bind: content, placeholder: "Write markdown..."
      "preview": rect padding: 16px, color: #f8fafc, radius: 8px {
        text "{content}" font-size: 16px
      }
      _: text "Unknown mode"
    }
  }
}""")

ex("gen-util-password-strength.naze", "Password strength meter with visual states",
   """-- Password strength
app "Password Strength" {
  state password = ""
  state strength = "empty"

  column padding: 20px, gap: 16px {
    heading "Password Strength"
    input bind: password, placeholder: "Enter password..."

    match strength {
      "empty": text "Enter a password" color: #94a3b8
      "weak": text "Weak" color: #dc2626, font-size: 20px
      "medium": text "Medium" color: #f59e0b, font-size: 20px
      "strong": text "Strong" color: #16a34a, font-size: 20px
      _: text "Unknown"
    }

    row gap: 8px {
      rect width: 80px, height: 32px, color: #dc2626, radius: 4px {
        text "Weak" color: #ffffff
        on click: set strength = "weak"
      }
      rect width: 80px, height: 32px, color: #f59e0b, radius: 4px {
        text "Medium" color: #ffffff
        on click: set strength = "medium"
      }
      rect width: 80px, height: 32px, color: #16a34a, radius: 4px {
        text "Strong" color: #ffffff
        on click: set strength = "strong"
      }
    }
  }
}""")

ex("gen-util-char-counter.naze", "Character counter for text input",
   """-- Character counter
app "Character Counter" {
  state text-input = ""
  state char-count = 0

  column padding: 20px, gap: 16px {
    heading "Character Counter"
    textarea bind: text-input, placeholder: "Type something..."
    text "{char-count} / 280 characters" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "Count" color: #ffffff
        on click: set char-count = char-count + 1
      }
      rect width: 80px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Clear"
        on click: set text-input = ""
      }
    }
  }
}""")

ex("gen-util-word-counter.naze", "Word counter with stats display",
   """-- Word counter
app "Word Counter" {
  state content = ""
  state words = 0
  state lines = 0

  column padding: 20px, gap: 16px {
    heading "Word Counter"
    textarea bind: content, placeholder: "Paste your text here..."

    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #eff6ff, radius: 8px {
        column gap: 4px {
          text "Words" color: #64748b, font-size: 12px
          text "{words}" font-size: 24px, color: #2563eb
        }
      }
      rect padding: 12px, color: #ecfdf5, radius: 8px {
        column gap: 4px {
          text "Lines" color: #64748b, font-size: 12px
          text "{lines}" font-size: 24px, color: #16a34a
        }
      }
      rect padding: 12px, color: #faf5ff, radius: 8px {
        column gap: 4px {
          text "Chars" color: #64748b, font-size: 12px
          text "0" font-size: 24px, color: #8b5cf6
        }
      }
    }

    rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
      text "Analyze" color: #ffffff
      on click: set words = words + 1
    }
  }
}""")

ex("gen-util-qr-display.naze", "QR code display with configurable URL",
   """-- QR code display
app "QR Display" {
  state url = "https://example.com"
  state size = 200

  column padding: 20px, gap: 16px {
    heading "QR Code Generator"
    input bind: url, placeholder: "Enter URL..."

    rect width: 200px, height: 200px, color: #f3f4f6, radius: 8px, padding: 20px {
      text "QR Code" font-size: 18px
      text "{url}" color: #64748b, font-size: 12px
    }

    text "Size: {size}px" color: #94a3b8

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "Small" color: #ffffff
        on click: set size = 150
      }
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "Large" color: #ffffff
        on click: set size = 300
      }
    }
  }
}""")

# ─── 8. Gaming/Fun patterns (7) ──────────────────────────────────────────────

ex("gen-game-scoreboard.naze", "Game scoreboard with player rankings",
   """-- Scoreboard
app "Scoreboard" {
  state players = [{name: "Alice", score: "2450"}, {name: "Bob", score: "2100"}, {name: "Carol", score: "1950"}, {name: "Dan", score: "1800"}]
  state round = 1
  computed player-count = players | count

  column padding: 20px, gap: 16px {
    heading "Scoreboard"
    text "Round {round}" color: #64748b
    text "{player-count} players" color: #94a3b8

    each p in players | sort-by score {
      row padding: 12px, color: #fef3c7, radius: 8px, gap: 12px {
        text "{p.name}" font-weight: bold
        text "{p.score} pts" color: #92400e, font-size: 18px
      }
    }

    rect width: 120px, height: 36px, color: #f59e0b, radius: 8px {
      text "Next Round" color: #ffffff
      on click: set round = round + 1
    }
  }
}""")

ex("gen-game-achievements.naze", "Achievement list with unlock tracking",
   """-- Achievements
app "Achievements" {
  state achievements = [{name: "First Steps", desc: "Complete tutorial"}, {name: "Explorer", desc: "Visit all areas"}, {name: "Champion", desc: "Win 10 matches"}, {name: "Collector", desc: "Find all items"}]
  state unlocked = 2
  computed total-achievements = achievements | count

  column padding: 20px, gap: 16px {
    heading "Achievements"
    text "{unlocked} / {total-achievements} unlocked" color: #f59e0b

    each ach in achievements {
      row padding: 12px, color: #fefce8, radius: 8px, gap: 8px {
        column gap: 2px {
          text "{ach.name}" font-weight: bold
          text "{ach.desc}" color: #64748b, font-size: 14px
        }
      }
    }
  }
}""")

ex("gen-game-inventory-grid.naze", "Game inventory grid with item slots",
   """-- Game inventory
app "Inventory" {
  state selected-slot = "none"

  column padding: 20px, gap: 16px {
    heading "Inventory"
    text "Selected: {selected-slot}" color: #64748b

    grid columns: 4, gap: 4px {
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Sword"
        on click: set selected-slot = "sword"
      }
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Shield"
        on click: set selected-slot = "shield"
      }
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Potion"
        on click: set selected-slot = "potion"
      }
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Key"
        on click: set selected-slot = "key"
      }
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Armor"
        on click: set selected-slot = "armor"
      }
      rect width: 70px, height: 70px, color: #e2e8f0, radius: 4px {
        text "Ring"
        on click: set selected-slot = "ring"
      }
      rect width: 70px, height: 70px, color: #f3f4f6, radius: 4px {
        text "Empty" color: #94a3b8
      }
      rect width: 70px, height: 70px, color: #f3f4f6, radius: 4px {
        text "Empty" color: #94a3b8
      }
    }
  }
}""")

ex("gen-game-character-stats.naze", "RPG character sheet with attribute allocation",
   """-- Character stats
app "Character Sheet" {
  state hp = 100
  state mp = 50
  state strength = 12
  state agility = 8
  state intel = 15
  state points = 5

  column padding: 20px, gap: 16px {
    heading "Character Sheet"

    row gap: 16px {
      text "HP: {hp}" color: #dc2626, font-size: 18px
      text "MP: {mp}" color: #2563eb, font-size: 18px
    }

    text "Points: {points}" color: #f59e0b

    column gap: 8px {
      row gap: 8px {
        text "STR: {strength}" font-weight: bold
        rect width: 40px, height: 28px, color: #dc2626, radius: 4px {
          text "+" color: #ffffff
          on click: set strength = strength + 1
        }
      }
      row gap: 8px {
        text "AGI: {agility}" font-weight: bold
        rect width: 40px, height: 28px, color: #16a34a, radius: 4px {
          text "+" color: #ffffff
          on click: set agility = agility + 1
        }
      }
      row gap: 8px {
        text "INT: {intel}" font-weight: bold
        rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
          text "+" color: #ffffff
          on click: set intel = intel + 1
        }
      }
    }
  }
}""")

ex("gen-game-level-select.naze", "Level selection screen with difficulty tiers",
   """-- Level select
app "Level Select" {
  state current-level = 1

  column padding: 20px, gap: 16px {
    heading "Select Level"
    text "Current: Level {current-level}" color: #6366f1, font-size: 18px

    grid columns: 3, gap: 8px {
      rect width: 80px, height: 60px, color: #ecfdf5, radius: 8px {
        text "Lv 1" font-weight: bold
        on click: set current-level = 1
      }
      rect width: 80px, height: 60px, color: #ecfdf5, radius: 8px {
        text "Lv 2" font-weight: bold
        on click: set current-level = 2
      }
      rect width: 80px, height: 60px, color: #fef3c7, radius: 8px {
        text "Lv 3" font-weight: bold
        on click: set current-level = 3
      }
      rect width: 80px, height: 60px, color: #fef3c7, radius: 8px {
        text "Lv 4" font-weight: bold
        on click: set current-level = 4
      }
      rect width: 80px, height: 60px, color: #fef2f2, radius: 8px {
        text "Lv 5" font-weight: bold
        on click: set current-level = 5
      }
      rect width: 80px, height: 60px, color: #fef2f2, radius: 8px {
        text "Boss" font-weight: bold, color: #dc2626
        on click: set current-level = 6
      }
    }
  }
}""")

ex("gen-game-menu.naze", "Game main menu with screen navigation",
   """-- Game menu
app "Space Quest" {
  state screen = "menu"

  column padding: 20px, gap: 16px {
    match screen {
      "menu": column gap: 16px, padding: 40px {
        heading "Space Quest" color: #6366f1, font-size: 32px
        text "A Galactic Adventure" color: #94a3b8

        rect width: 200px, height: 50px, color: #6366f1, radius: 8px {
          text "New Game" color: #ffffff, font-size: 18px
          on click: set screen = "play"
        }
        rect width: 200px, height: 50px, color: #64748b, radius: 8px {
          text "Settings" color: #ffffff, font-size: 18px
          on click: set screen = "settings"
        }
      }
      "play": column gap: 16px {
        heading "Playing..."
        text "Game world here" color: #64748b
        rect width: 100px, height: 36px, color: #dc2626, radius: 8px {
          text "Menu" color: #ffffff
          on click: set screen = "menu"
        }
      }
      "settings": column gap: 16px {
        heading "Settings"
        text "Volume, controls" color: #64748b
        rect width: 100px, height: 36px, color: #64748b, radius: 8px {
          text "Back" color: #ffffff
          on click: set screen = "menu"
        }
      }
      _: text "Unknown screen"
    }
  }
}""")

ex("gen-game-dice-roller.naze", "Dice roller with total and roll counter",
   """-- Dice roller
app "Dice Roller" {
  state d1 = 1
  state d2 = 1
  state rolls = 0
  computed dice-total = d1 + d2

  column padding: 20px, gap: 16px {
    heading "Dice Roller"

    row gap: 16px {
      rect width: 80px, height: 80px, color: #ffffff, radius: 8px {
        text "{d1}" font-size: 36px, color: #1e293b
      }
      rect width: 80px, height: 80px, color: #ffffff, radius: 8px {
        text "{d2}" font-size: 36px, color: #1e293b
      }
    }

    text "Total: {dice-total}" font-size: 24px, color: #2563eb
    text "Rolls: {rolls}" color: #94a3b8

    row gap: 8px {
      rect width: 100px, height: 44px, color: #dc2626, radius: 8px {
        text "Roll!" color: #ffffff, font-size: 18px
        on click: set rolls = rolls + 1
      }
      rect width: 100px, height: 44px, color: #e2e8f0, radius: 8px {
        text "Reset"
        on click: set rolls = 0
      }
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# COMPLEXITY VARIATIONS & FEATURE COMBINATIONS (75 examples)
# ═══════════════════════════════════════════════════════════════════════════════

# ─── Category 1: Minimal apps (5-8 lines each) ──────────────────────────────
# 15 examples teaching the minimal valid .naze structure

MINIMAL_T = """-- __DESC__
app "__TITLE__" {
  column padding: __PAD__px {
    __BODY__
  }
}"""

for n, cfg in [
    ("hello-world", {
        "TITLE": "Hello World", "DESC": "The simplest possible hello world app",
        "PAD": "20", "BODY": 'heading "Hello, World!"',
    }),
    ("single-text", {
        "TITLE": "Just Text", "DESC": "App displaying a single styled text element",
        "PAD": "16", "BODY": 'text "Naze is declarative" color: #2563eb, font-size: 18px',
    }),
    ("bare-heading", {
        "TITLE": "Title Only", "DESC": "App with only a heading and nothing else",
        "PAD": "24", "BODY": 'heading "Welcome" font-size: 32px, color: #1e293b',
    }),
    ("static-pair", {
        "TITLE": "Greeting", "DESC": "Heading and one line of descriptive text",
        "PAD": "20", "BODY": 'heading "Good Morning"\n    text "Have a great day" color: #64748b',
    }),
    ("single-image", {
        "TITLE": "Photo", "DESC": "App showing a single image element",
        "PAD": "16", "BODY": 'image src: "/photo.jpg", width: 300px',
    }),
    ("empty-rect", {
        "TITLE": "Box", "DESC": "A single colored rectangle with rounded corners",
        "PAD": "20", "BODY": 'rect width: 200px, height: 200px, color: #6366f1, radius: 16px',
    }),
    ("separator-demo", {
        "TITLE": "Divider", "DESC": "Two headings separated by a divider line",
        "PAD": "20", "BODY": 'heading "Above"\n    separator\n    heading "Below"',
    }),
    ("spacer-demo", {
        "TITLE": "Spaced", "DESC": "Heading and text with a spacer between them",
        "PAD": "20", "BODY": 'heading "Top"\n    spacer height: 40px\n    text "Bottom" color: #94a3b8',
    }),
    ("link-only", {
        "TITLE": "Links", "DESC": "A minimal link to an external URL",
        "PAD": "20", "BODY": 'link "Visit Example" href: "https://example.com"',
    }),
]:
    ex(f"gen-min-{n}.naze", cfg["DESC"], fill(MINIMAL_T, cfg))

# Minimal apps that need state (still very short)

ex("gen-min-counter.naze",
   "The smallest possible counter app",
   """-- Minimal counter
app "Count" {
  state n = 0
  column padding: 20px {
    text "{n}" font-size: 48px
    rect width: 60px, height: 36px, color: #2563eb, radius: 4px {
      text "+" color: #ffffff
      on click: set n = n + 1
    }
  }
}""")

ex("gen-min-toggle.naze",
   "A minimal boolean toggle",
   """-- Minimal toggle
app "Toggle" {
  state on = false
  column padding: 20px {
    text "State: {on}" font-size: 20px
    rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
      text "Toggle" color: #ffffff
      on click: set on = true
    }
  }
}""")

ex("gen-min-input.naze",
   "A minimal input that echoes what you type",
   """-- Minimal input echo
app "Echo" {
  state msg = ""
  column padding: 20px, gap: 12px {
    input bind: msg, placeholder: "Type here..."
    text "You typed: {msg}"
  }
}""")

ex("gen-min-checkbox.naze",
   "A single checkbox with label",
   """-- Minimal checkbox
app "Agree" {
  state agreed = false
  column padding: 20px, gap: 12px {
    checkbox bind: agreed, label: "I agree to the terms"
    text "Agreed: {agreed}" color: #64748b
  }
}""")

ex("gen-min-computed.naze",
   "Minimal app with one computed value",
   """-- Minimal computed
app "Double" {
  state x = 5
  computed doubled = x * 2
  column padding: 20px, gap: 8px {
    text "Value: {x}" font-size: 20px
    text "Doubled: {doubled}" color: #2563eb, font-size: 20px
  }
}""")

ex("gen-min-row.naze",
   "Minimal horizontal row layout with three colored boxes",
   """-- Minimal row
app "Boxes" {
  row padding: 16px, gap: 8px {
    rect width: 60px, height: 60px, color: #ef4444, radius: 8px
    rect width: 60px, height: 60px, color: #22c55e, radius: 8px
    rect width: 60px, height: 60px, color: #3b82f6, radius: 8px
  }
}""")

# ─── Category 2: Medium apps with state + UI ────────────────────────────────
# 15 examples combining 2-3 features

ex("gen-med-counter-reset.naze",
   "Counter with increment, decrement, and reset buttons",
   """-- Counter with controls
app "Counter Pro" {
  state count = 0
  computed is-positive = count

  column padding: 20px, gap: 16px {
    heading "Counter" color: #1e293b
    text "{count}" font-size: 48px, color: #2563eb

    row gap: 8px {
      rect width: 60px, height: 40px, color: #16a34a, radius: 8px {
        text "+" color: #ffffff
        on click: set count = count + 1
      }
      rect width: 60px, height: 40px, color: #ef4444, radius: 8px {
        text "-" color: #ffffff
        on click: set count = count - 1
      }
      rect width: 80px, height: 40px, color: #64748b, radius: 8px {
        text "Reset" color: #ffffff
        on click: set count = 0
      }
    }
  }
}""")

ex("gen-med-visibility.naze",
   "Toggle visibility of a content section",
   """-- Toggle visibility
app "Visibility" {
  state visible = true

  column padding: 20px, gap: 16px {
    heading "Toggle Demo"

    rect width: 120px, height: 40px, color: #6366f1, radius: 8px {
      text "Toggle" color: #ffffff
      on click: set visible = false
    }

    if visible {
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "This content is visible!" color: #16a34a
      }
    }

    if visible == false {
      text "Content is hidden" color: #94a3b8
    }
  }
}""")

ex("gen-med-tab-switch.naze",
   "Two-tab switcher with content panels",
   """-- Tab switcher
app "Tabs Demo" {
  state tab = "info"

  column padding: 20px, gap: 16px {
    heading "Two Tabs"

    row gap: 8px {
      rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
        text "Info" color: #ffffff
        on click: set tab = "info"
      }
      rect width: 100px, height: 40px, color: #8b5cf6, radius: 8px {
        text "Settings" color: #ffffff
        on click: set tab = "settings"
      }
    }

    match tab {
      "info": column gap: 8px {
        text "Information panel" font-weight: bold
        text "Here you find general info" color: #64748b
      }
      "settings": column gap: 8px {
        text "Settings panel" font-weight: bold
        text "Adjust your preferences" color: #64748b
      }
      _: text "Unknown tab"
    }
  }
}""")

ex("gen-med-accordion.naze",
   "Accordion with two expandable sections",
   """-- Accordion
app "Accordion" {
  state open = "none"

  column padding: 20px, gap: 4px {
    heading "FAQ"

    rect padding: 12px, color: #f1f5f9, radius: 4px {
      text "What is Naze?" font-weight: bold
      on click: set open = "a"
    }
    if open == "a" {
      rect padding: 12px, color: #eff6ff {
        text "A declarative UI language compiled to WASM." color: #374151
      }
    }

    rect padding: 12px, color: #f1f5f9, radius: 4px {
      text "How fast is it?" font-weight: bold
      on click: set open = "b"
    }
    if open == "b" {
      rect padding: 12px, color: #eff6ff {
        text "Very fast. It renders via Canvas2D." color: #374151
      }
    }
  }
}""")

ex("gen-med-rating.naze",
   "Star rating selector from 1 to 5",
   """-- Star rating
app "Rate Us" {
  state rating = 0

  column padding: 20px, gap: 16px {
    heading "How would you rate us?"
    text "Rating: {rating} / 5" font-size: 20px, color: #f59e0b

    row gap: 4px {
      rect width: 50px, height: 50px, color: #fef3c7, radius: 8px {
        text "1"
        on click: set rating = 1
      }
      rect width: 50px, height: 50px, color: #fef3c7, radius: 8px {
        text "2"
        on click: set rating = 2
      }
      rect width: 50px, height: 50px, color: #fef3c7, radius: 8px {
        text "3"
        on click: set rating = 3
      }
      rect width: 50px, height: 50px, color: #fef3c7, radius: 8px {
        text "4"
        on click: set rating = 4
      }
      rect width: 50px, height: 50px, color: #fef3c7, radius: 8px {
        text "5"
        on click: set rating = 5
      }
    }
  }
}""")

ex("gen-med-like-btn.naze",
   "Like button with count and visual feedback",
   """-- Like button
app "Like" {
  state likes = 42
  state liked = false

  column padding: 20px, gap: 16px {
    heading "Post Title"
    text "Some interesting content here." color: #374151

    row gap: 8px {
      rect width: 100px, height: 40px, color: #ef4444, radius: 8px {
        text "Like ({likes})" color: #ffffff
        on click: set likes = likes + 1
      }
    }

    if liked {
      text "You liked this!" color: #ef4444
    }
  }
}""")

ex("gen-med-bookmark.naze",
   "Bookmark toggle with saved state indicator",
   """-- Bookmark toggle
app "Bookmarker" {
  state saved = false
  state count = 7

  column padding: 20px, gap: 16px {
    heading "Article Title"
    text "Read this amazing article..." color: #374151

    row gap: 8px {
      rect width: 120px, height: 40px, color: #f59e0b, radius: 8px {
        text "Bookmark" color: #ffffff
        on click: set saved = true
      }
      rect width: 120px, height: 40px, color: #e2e8f0, radius: 8px {
        text "Unbookmark"
        on click: set saved = false
      }
    }

    if saved {
      text "Bookmarked!" color: #f59e0b, font-weight: bold
    }
  }
}""")

ex("gen-med-mode-switch.naze",
   "Switch between edit and preview mode",
   """-- Mode switcher
app "Editor" {
  state mode = "edit"
  state content = "Hello world"

  column padding: 20px, gap: 16px {
    heading "Document"

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "Edit" color: #ffffff
        on click: set mode = "edit"
      }
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Preview" color: #ffffff
        on click: set mode = "preview"
      }
    }

    match mode {
      "edit": input bind: content, placeholder: "Write here..."
      "preview": text "{content}" font-size: 18px, color: #1e293b
      _: text "Unknown mode"
    }
  }
}""")

ex("gen-med-lang-select.naze",
   "Language selector that changes greeting text",
   """-- Language selector
app "i18n Demo" {
  state lang = "en"

  column padding: 20px, gap: 16px {
    heading "Language"

    match lang {
      "en": text "Hello!" font-size: 24px
      "es": text "Hola!" font-size: 24px
      "ja": text "Konnichiwa!" font-size: 24px
      _: text "Select a language"
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "EN" color: #ffffff
        on click: set lang = "en"
      }
      rect width: 80px, height: 36px, color: #ef4444, radius: 4px {
        text "ES" color: #ffffff
        on click: set lang = "es"
      }
      rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
        text "JA" color: #ffffff
        on click: set lang = "ja"
      }
    }
  }
}""")

ex("gen-med-sort-toggle.naze",
   "Toggle sort direction for a list display",
   """-- Sort toggle
app "Sort Demo" {
  state direction = "asc"
  state items = [{name: "Banana"}, {name: "Apple"}, {name: "Cherry"}]

  column padding: 20px, gap: 16px {
    heading "Sorted List"
    text "Direction: {direction}" color: #64748b

    rect width: 120px, height: 36px, color: #6366f1, radius: 4px {
      text "Toggle Sort" color: #ffffff
      on click: set direction = "desc"
    }

    each item in items | sort-by name {
      text "{item.name}" font-size: 16px
    }
  }
}""")

ex("gen-med-view-toggle.naze",
   "Toggle between list view and grid view",
   """-- View mode toggle
app "Gallery" {
  state view = "grid"

  column padding: 20px, gap: 16px {
    heading "Photo Gallery"

    row gap: 8px {
      rect width: 80px, height: 36px, color: #3b82f6, radius: 4px {
        text "Grid" color: #ffffff
        on click: set view = "grid"
      }
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "List" color: #ffffff
        on click: set view = "list"
      }
    }

    match view {
      "grid": grid columns: 3, gap: 8px {
        rect width: 80px, height: 80px, color: #e2e8f0, radius: 8px
        rect width: 80px, height: 80px, color: #e2e8f0, radius: 8px
        rect width: 80px, height: 80px, color: #e2e8f0, radius: 8px
      }
      "list": column gap: 4px {
        rect width: 260px, height: 40px, color: #e2e8f0, radius: 4px
        rect width: 260px, height: 40px, color: #e2e8f0, radius: 4px
        rect width: 260px, height: 40px, color: #e2e8f0, radius: 4px
      }
      _: text "Unknown view"
    }
  }
}""")

ex("gen-med-volume-ctrl.naze",
   "Volume control with level indicator",
   """-- Volume control
app "Volume" {
  state vol = 50
  computed level = vol

  column padding: 20px, gap: 16px {
    heading "Volume Control"
    text "Level: {vol}%" font-size: 24px, color: #2563eb

    row gap: 8px {
      rect width: 60px, height: 40px, color: #2563eb, radius: 8px {
        text "+" color: #ffffff
        on click: set vol = vol + 10
      }
      rect width: 60px, height: 40px, color: #64748b, radius: 8px {
        text "-" color: #ffffff
        on click: set vol = vol - 10
      }
      rect width: 80px, height: 40px, color: #ef4444, radius: 8px {
        text "Mute" color: #ffffff
        on click: set vol = 0
      }
    }
  }
}""")

ex("gen-med-brightness.naze",
   "Brightness adjuster with percentage display",
   """-- Brightness adjuster
app "Brightness" {
  state brightness = 75

  column padding: 20px, gap: 16px {
    heading "Display Settings"
    text "Brightness: {brightness}%" font-size: 20px

    row gap: 8px {
      rect width: 100px, height: 40px, color: #f59e0b, radius: 8px {
        text "Brighter" color: #ffffff
        on click: set brightness = brightness + 5
      }
      rect width: 100px, height: 40px, color: #334155, radius: 8px {
        text "Dimmer" color: #ffffff
        on click: set brightness = brightness - 5
      }
    }
  }
}""")

ex("gen-med-select-dropdown.naze",
   "Dropdown select with option display",
   """-- Select dropdown
app "Picker" {
  state choice = "none"

  column padding: 20px, gap: 16px {
    heading "Choose a Color"
    select bind: choice {
      option "Red" value: "red"
      option "Green" value: "green"
      option "Blue" value: "blue"
    }
    text "You chose: {choice}" color: #2563eb, font-size: 18px
  }
}""")

ex("gen-med-step-wizard.naze",
   "Two-step wizard with back and next navigation",
   """-- Step wizard
app "Wizard" {
  state step = 1

  column padding: 20px, gap: 16px {
    heading "Setup Wizard"
    text "Step {step} of 2" color: #64748b

    match step {
      1: column gap: 12px {
        text "Welcome! Let's get started." font-weight: bold
        rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
          text "Next" color: #ffffff
          on click: set step = 2
        }
      }
      2: column gap: 12px {
        text "All done! You're ready." font-weight: bold
        rect width: 80px, height: 36px, color: #64748b, radius: 4px {
          text "Back" color: #ffffff
          on click: set step = 1
        }
      }
      _: text "Unknown step"
    }
  }
}""")

# ─── Category 3: Complex apps (30-50 lines each) ────────────────────────────
# 15 examples: full-featured applications

ex("gen-cx-expense-tracker.naze",
   "Expense tracker with categories, totals, and add form",
   """-- Expense tracker
app "Expenses" {
  state desc = ""
  state amount = 0
  state expenses = [{desc: "Groceries", amount: "85", category: "food"}, {desc: "Gas", amount: "45", category: "transport"}, {desc: "Netflix", amount: "15", category: "entertainment"}]
  computed total = expenses | count
  computed food-items = expenses | filter category == "food"

  column padding: 20px, gap: 16px {
    heading "Expense Tracker"
    text "{total} expenses recorded" color: #64748b

    row gap: 8px {
      input bind: desc, placeholder: "Description"
      input bind: amount, placeholder: "Amount"
      rect width: 80px, height: 40px, color: #16a34a, radius: 8px {
        text "Add" color: #ffffff
        on click: set desc = ""
      }
    }

    separator

    heading "All Expenses" font-size: 18px
    each exp in expenses | sort-by category {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        column gap: 2px {
          text "{exp.desc}" font-weight: bold
          text "{exp.category}" color: #94a3b8, font-size: 12px
        }
        text "${exp.amount}" color: #2563eb, font-size: 18px
      }
    }
  }
}""")

ex("gen-cx-multi-tab-dash.naze",
   "Multi-tab dashboard with overview, analytics, and settings tabs",
   """-- Multi-tab dashboard
app "Admin Panel" {
  state tab = "overview"
  state users = 1420
  state revenue = 34500
  state orders = 287
  computed avg = revenue / orders

  column padding: 20px, gap: 16px {
    heading "Admin Dashboard" color: #1e293b

    row gap: 8px {
      rect width: 100px, height: 36px, color: #2563eb, radius: 4px {
        text "Overview" color: #ffffff
        on click: set tab = "overview"
      }
      rect width: 100px, height: 36px, color: #8b5cf6, radius: 4px {
        text "Analytics" color: #ffffff
        on click: set tab = "analytics"
      }
      rect width: 100px, height: 36px, color: #64748b, radius: 4px {
        text "Settings" color: #ffffff
        on click: set tab = "settings"
      }
    }

    separator

    match tab {
      "overview": column gap: 12px {
        grid columns: 2, gap: 12px {
          rect padding: 16px, color: #eff6ff, radius: 8px {
            text "Users" color: #64748b, font-size: 12px
            text "{users}" font-size: 24px, color: #1e40af
          }
          rect padding: 16px, color: #f0fdf4, radius: 8px {
            text "Revenue" color: #64748b, font-size: 12px
            text "${revenue}" font-size: 24px, color: #166534
          }
        }
      }
      "analytics": column gap: 12px {
        text "Orders: {orders}" font-size: 18px
        text "Avg order: ${avg}" font-size: 18px, color: #2563eb
      }
      "settings": column gap: 12px {
        text "Admin Settings" font-weight: bold
        text "Configure your dashboard" color: #64748b
      }
      _: text "Unknown"
    }
  }
}""")

ex("gen-cx-project-mgr.naze",
   "Project manager with tasks, status filters, and progress",
   """-- Project manager
app "Projects" {
  state filter = "all"
  state tasks = [{title: "Design mockups", status: "done", priority: "high"}, {title: "Build API", status: "progress", priority: "high"}, {title: "Write tests", status: "todo", priority: "medium"}, {title: "Deploy staging", status: "todo", priority: "low"}, {title: "Code review", status: "progress", priority: "medium"}]
  computed task-count = tasks | count

  column padding: 20px, gap: 16px {
    heading "Project Alpha"
    text "{task-count} tasks total" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 32px, color: #334155, radius: 4px {
        text "All" color: #ffffff
        on click: set filter = "all"
      }
      rect width: 80px, height: 32px, color: #16a34a, radius: 4px {
        text "Done" color: #ffffff
        on click: set filter = "done"
      }
      rect width: 100px, height: 32px, color: #f59e0b, radius: 4px {
        text "In Progress" color: #ffffff
        on click: set filter = "progress"
      }
      rect width: 60px, height: 32px, color: #dc2626, radius: 4px {
        text "Todo" color: #ffffff
        on click: set filter = "todo"
      }
    }

    separator

    each task in tasks | sort-by priority {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 8px {
        column gap: 2px {
          text "{task.title}" font-weight: bold
          row gap: 8px {
            text "{task.status}" color: #2563eb, font-size: 12px
            text "{task.priority}" color: #94a3b8, font-size: 12px
          }
        }
      }
    }
  }
}""")

ex("gen-cx-weather-cities.naze",
   "Weather display for multiple cities with temperature data",
   """-- Weather for cities
app "Weather" {
  state selected = "london"
  state cities = [{name: "London", temp: "12", condition: "Cloudy"}, {name: "Tokyo", temp: "22", condition: "Sunny"}, {name: "New York", temp: "8", condition: "Rainy"}, {name: "Sydney", temp: "28", condition: "Clear"}]
  computed city-count = cities | count

  column padding: 20px, gap: 16px {
    heading "World Weather"
    text "{city-count} cities tracked" color: #64748b

    each city in cities | sort-by name {
      row padding: 12px, color: #f0f9ff, radius: 8px, gap: 12px {
        column gap: 4px {
          text "{city.name}" font-weight: bold, font-size: 18px
          text "{city.condition}" color: #64748b
        }
        text "{city.temp}C" font-size: 28px, color: #0ea5e9
      }
    }
  }
}""")

ex("gen-cx-recipe-app.naze",
   "Recipe app with ingredients list, step display, and serving adjuster",
   """-- Recipe app
app "Recipe Book" {
  state servings = 4
  state active-step = 1
  state ingredients = [{name: "Flour", qty: "2 cups"}, {name: "Eggs", qty: "3"}, {name: "Butter", qty: "100g"}, {name: "Sugar", qty: "1 cup"}, {name: "Vanilla", qty: "1 tsp"}]
  computed ing-count = ingredients | count

  column padding: 20px, gap: 16px {
    heading "Chocolate Cake Recipe"
    text "Servings: {servings}" color: #64748b

    row gap: 8px {
      rect width: 40px, height: 32px, color: #2563eb, radius: 4px {
        text "-" color: #ffffff
        on click: set servings = servings - 1
      }
      rect width: 40px, height: 32px, color: #2563eb, radius: 4px {
        text "+" color: #ffffff
        on click: set servings = servings + 1
      }
    }

    separator

    text "Ingredients ({ing-count}):" font-weight: bold
    each ing in ingredients {
      row padding: 4px, gap: 8px {
        text "{ing.qty}" color: #2563eb, font-weight: bold
        text "{ing.name}" color: #374151
      }
    }

    separator

    text "Step {active-step}:" font-weight: bold
    match active-step {
      1: text "Preheat oven to 180C" color: #374151
      2: text "Mix dry ingredients" color: #374151
      3: text "Add wet ingredients and combine" color: #374151
      _: text "Bake for 30 minutes"
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "Prev" color: #ffffff
        on click: set active-step = active-step - 1
      }
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Next" color: #ffffff
        on click: set active-step = active-step + 1
      }
    }
  }
}""")

ex("gen-cx-workout-log.naze",
   "Workout log with exercises, sets, and session timer",
   """-- Workout log
app "Workout" {
  state elapsed = 0
  state exercises = [{name: "Squats", sets: "3", reps: "12"}, {name: "Bench Press", sets: "4", reps: "8"}, {name: "Deadlift", sets: "3", reps: "5"}, {name: "Pull-ups", sets: "3", reps: "10"}]
  computed exercise-count = exercises | count

  timer session: every 1s {
    set elapsed = elapsed + 1
  }

  column padding: 20px, gap: 16px {
    heading "Today's Workout"
    row gap: 16px {
      text "{elapsed}s" font-size: 24px, color: #2563eb
      text "{exercise-count} exercises" color: #64748b
    }

    each ex in exercises {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 12px {
        column gap: 4px {
          text "{ex.name}" font-weight: bold, font-size: 16px
          row gap: 8px {
            text "{ex.sets} sets" color: #64748b
            text "{ex.reps} reps" color: #64748b
          }
        }
      }
    }
  }
}""")

ex("gen-cx-blog-editor.naze",
   "Blog editor with title, body, preview mode, and publish button",
   """-- Blog editor
app "Blog Editor" {
  state title = ""
  state body = ""
  state mode = "write"
  state published = false

  column padding: 20px, gap: 16px {
    heading "New Blog Post"

    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "Write" color: #ffffff
        on click: set mode = "write"
      }
      rect width: 80px, height: 36px, color: #8b5cf6, radius: 4px {
        text "Preview" color: #ffffff
        on click: set mode = "preview"
      }
    }

    match mode {
      "write": column gap: 12px {
        input bind: title, placeholder: "Post title..."
        textarea bind: body, placeholder: "Write your post..."
      }
      "preview": column gap: 12px {
        heading "{title}" font-size: 24px
        separator
        text "{body}" color: #374151
      }
      _: text "Unknown mode"
    }

    if published == false {
      rect width: 120px, height: 40px, color: #16a34a, radius: 8px {
        text "Publish" color: #ffffff
        on click: set published = true
      }
    }

    if published {
      text "Published successfully!" color: #16a34a, font-weight: bold
    }
  }
}""")

ex("gen-cx-survey-builder.naze",
   "Survey with questions, progress tracking, and submission",
   """-- Survey
app "Survey" {
  state step = 1
  state answer1 = ""
  state answer2 = ""
  state submitted = false
  computed progress = step * 50

  column padding: 20px, gap: 16px {
    heading "Customer Survey"
    text "Progress: {progress}%" color: #64748b

    if submitted == false {
      match step {
        1: column gap: 12px {
          text "How did you hear about us?" font-weight: bold
          input bind: answer1, placeholder: "Your answer..."
          rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
            text "Next" color: #ffffff
            on click: set step = 2
          }
        }
        2: column gap: 12px {
          text "Any suggestions?" font-weight: bold
          textarea bind: answer2, placeholder: "Your suggestions..."
          rect width: 100px, height: 40px, color: #16a34a, radius: 8px {
            text "Submit" color: #ffffff
            on click: set submitted = true
          }
        }
        _: text "Complete"
      }
    }

    if submitted {
      text "Thank you for your feedback!" color: #16a34a, font-size: 20px
    }
  }
}""")

ex("gen-cx-portfolio.naze",
   "Portfolio page with projects grid and about section",
   """-- Portfolio
app "Portfolio" {
  state section = "projects"
  state projects = [{title: "Naze App", tech: "Naze"}, {title: "API Server", tech: "Rust"}, {title: "ML Model", tech: "Python"}, {title: "Mobile App", tech: "Swift"}]
  computed project-count = projects | count

  column padding: 20px, gap: 16px {
    heading "Jane Doe" font-size: 28px
    text "Full-Stack Developer" color: #64748b

    row gap: 8px {
      rect width: 100px, height: 36px, color: #2563eb, radius: 4px {
        text "Projects" color: #ffffff
        on click: set section = "projects"
      }
      rect width: 100px, height: 36px, color: #64748b, radius: 4px {
        text "About" color: #ffffff
        on click: set section = "about"
      }
    }

    match section {
      "projects": column gap: 12px {
        text "{project-count} projects" color: #94a3b8
        grid columns: 2, gap: 12px {
          each proj in projects {
            rect padding: 16px, color: #f8fafc, radius: 8px {
              column gap: 4px {
                text "{proj.title}" font-weight: bold
                text "{proj.tech}" color: #2563eb, font-size: 12px
              }
            }
          }
        }
      }
      "about": column gap: 12px {
        text "About Me" font-weight: bold, font-size: 20px
        text "I build things for the web." color: #374151
      }
      _: text "Unknown"
    }
  }
}""")

ex("gen-cx-team-roster.naze",
   "Team roster with members, roles, and headcount",
   """-- Team roster
app "Team" {
  state members = [{name: "Alice", role: "Lead", dept: "engineering"}, {name: "Bob", role: "Designer", dept: "design"}, {name: "Carol", role: "PM", dept: "product"}, {name: "Dan", role: "Engineer", dept: "engineering"}]
  computed headcount = members | count

  column padding: 20px, gap: 16px {
    heading "Team Directory"
    text "{headcount} members" color: #64748b

    each member in members | sort-by name {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        column gap: 2px {
          text "{member.name}" font-weight: bold
          row gap: 8px {
            text "{member.role}" color: #2563eb, font-size: 12px
            text "{member.dept}" color: #94a3b8, font-size: 12px
          }
        }
      }
    }
  }
}""")

ex("gen-cx-music-lib.naze",
   "Music library with song list and play controls",
   """-- Music library
app "Music" {
  state playing = "Nothing"
  state songs = [{title: "Bohemian Rhapsody", artist: "Queen"}, {title: "Imagine", artist: "Lennon"}, {title: "Yesterday", artist: "Beatles"}]
  computed song-count = songs | count

  column padding: 20px, gap: 16px {
    heading "My Music"
    text "Now playing: {playing}" color: #2563eb
    text "{song-count} songs" color: #64748b

    each song in songs | sort-by artist {
      row padding: 12px, color: #faf5ff, radius: 8px, gap: 12px {
        column gap: 2px {
          text "{song.title}" font-weight: bold
          text "{song.artist}" color: #8b5cf6, font-size: 12px
        }
        rect width: 60px, height: 32px, color: #8b5cf6, radius: 4px {
          text "Play" color: #ffffff
          on click: set playing = "Playing"
        }
      }
    }
  }
}""")

ex("gen-cx-contact-book.naze",
   "Contact book with add form and sorted display",
   """-- Contact book
app "Contacts" {
  state new-name = ""
  state contacts = [{name: "Alice Smith", phone: "555-0101"}, {name: "Bob Jones", phone: "555-0202"}, {name: "Carol White", phone: "555-0303"}]
  computed total = contacts | count

  column padding: 20px, gap: 16px {
    heading "Contact Book"
    text "{total} contacts" color: #64748b

    row gap: 8px {
      input bind: new-name, placeholder: "Name"
      rect width: 60px, height: 36px, color: #16a34a, radius: 4px {
        text "Add" color: #ffffff
        on click: set new-name = ""
      }
    }

    each contact in contacts | sort-by name {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        text "{contact.name}" font-weight: bold
        text "{contact.phone}" color: #2563eb
      }
    }
  }
}""")

ex("gen-cx-shopping-list.naze",
   "Shopping list with budget tracking and sorted items",
   """-- Shopping list
app "Shopping" {
  state budget = 100
  state spent = 0
  state items = [{name: "Milk", price: "4", category: "dairy"}, {name: "Bread", price: "3", category: "bakery"}, {name: "Chicken", price: "12", category: "meat"}]
  computed remaining = budget - spent
  computed item-count = items | count

  column padding: 20px, gap: 16px {
    heading "Shopping List"
    row gap: 16px {
      text "Budget: ${budget}" color: #64748b
      text "Remaining: ${remaining}" color: #16a34a
      text "{item-count} items" color: #94a3b8
    }

    each item in items | sort-by category {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{item.name}" font-weight: bold
        text "${item.price}" color: #2563eb
        text "{item.category}" color: #94a3b8, font-size: 12px
      }
    }
  }
}""")

ex("gen-cx-event-planner.naze",
   "Event planner with schedule and venue tabs",
   """-- Event planner
app "Event Planner" {
  state rsvp-count = 45
  state max-capacity = 100
  state tab = "schedule"
  computed spots-left = max-capacity - rsvp-count
  state schedule = [{time: "9 AM", session: "Registration"}, {time: "10 AM", session: "Keynote"}, {time: "12 PM", session: "Lunch"}]

  column padding: 20px, gap: 16px {
    heading "Annual Meetup"
    row gap: 16px {
      text "RSVPs: {rsvp-count}" color: #2563eb
      text "Spots left: {spots-left}" color: #16a34a
    }

    row gap: 8px {
      rect width: 80px, height: 32px, color: #2563eb, radius: 4px {
        text "Schedule" color: #ffffff
        on click: set tab = "schedule"
      }
      rect width: 80px, height: 32px, color: #64748b, radius: 4px {
        text "Venue" color: #ffffff
        on click: set tab = "venue"
      }
    }

    match tab {
      "schedule": column gap: 8px {
        each item in schedule {
          row padding: 8px, color: #f0f9ff, radius: 4px, gap: 12px {
            text "{item.time}" font-weight: bold, color: #2563eb
            text "{item.session}" color: #374151
          }
        }
      }
      "venue": column gap: 8px {
        text "Venue: Convention Center" font-weight: bold
        text "Capacity: {max-capacity}" color: #64748b
      }
      _: text "Unknown"
    }
  }
}""")

ex("gen-cx-invoice.naze",
   "Invoice with line items and tax rate",
   """-- Invoice
app "Invoice" {
  state items = [{desc: "Web Design", qty: "1", rate: "500"}, {desc: "Development", qty: "3", rate: "150"}, {desc: "Hosting", qty: "12", rate: "10"}]
  state tax-rate = 10
  computed line-count = items | count

  column padding: 20px, gap: 16px {
    heading "Invoice #1042"
    text "To: Acme Corp" color: #64748b

    separator

    each item in items {
      row padding: 8px, gap: 16px {
        text "{item.desc}"
        text "x{item.qty}" color: #64748b
        text "${item.rate}" color: #2563eb
      }
    }

    separator

    text "{line-count} line items" color: #94a3b8
    text "Tax rate: {tax-rate}%" color: #64748b
  }
}""")

# ─── Category 4: State + computed + pipeline combos ─────────────────────────
# 10 examples heavily using computed values and pipeline operations

ex("gen-pipe-filtered-stats.naze",
   "Student list with filtered counts and top-N display",
   """-- Filtered student stats
app "Class Stats" {
  state students = [{name: "Alice", grade: 92}, {name: "Bob", grade: 67}, {name: "Carol", grade: 85}, {name: "Dan", grade: 45}, {name: "Eve", grade: 78}]
  computed total = students | count
  computed passing = students | filter grade > 60 | count
  computed honors = students | filter grade > 85 | count

  column padding: 20px, gap: 16px {
    heading "Class Statistics"
    row gap: 16px {
      text "Total: {total}" color: #64748b
      text "Passing: {passing}" color: #16a34a
      text "Honors: {honors}" color: #2563eb
    }

    text "Top Students:" font-weight: bold
    each s in students | sort-by grade | take 3 {
      row padding: 8px, color: #f0fdf4, radius: 4px, gap: 8px {
        text "{s.name}" font-weight: bold
        text "{s.grade}" color: #16a34a
      }
    }
  }
}""")

ex("gen-pipe-sales-agg.naze",
   "Sales data with aggregated totals and sorted performers",
   """-- Sales aggregation
app "Sales Report" {
  state sales = [{rep: "Alice", amount: 12000, region: "north"}, {rep: "Bob", amount: 8500, region: "south"}, {rep: "Carol", amount: 15000, region: "north"}]
  computed rep-count = sales | count
  computed total-amount = sales | map amount | sum

  column padding: 20px, gap: 16px {
    heading "Sales Report"
    text "{rep-count} reps, Total: ${total-amount}" color: #16a34a

    each sale in sales | sort-by amount {
      row padding: 8px, color: #f0fdf4, radius: 4px, gap: 12px {
        text "{sale.rep}" font-weight: bold
        text "${sale.amount}" color: #2563eb
        text "{sale.region}" color: #94a3b8
      }
    }
  }
}""")

ex("gen-pipe-inventory-analysis.naze",
   "Inventory with low-stock alerts and sorted quantities",
   """-- Inventory analysis
app "Stock Check" {
  state products = [{name: "Widget A", qty: 150, price: 10}, {name: "Widget B", qty: 3, price: 25}, {name: "Widget C", qty: 45, price: 15}]
  computed total-products = products | count
  computed low-stock = products | filter qty < 10 | count

  column padding: 20px, gap: 16px {
    heading "Inventory Analysis"
    row gap: 16px {
      text "Products: {total-products}" color: #64748b
      text "Low stock: {low-stock}" color: #ef4444
    }

    each p in products | sort-by qty {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{p.name}" font-weight: bold
        text "Qty: {p.qty}" color: #2563eb
        text "${p.price}" color: #64748b
      }
    }
  }
}""")

ex("gen-pipe-task-metrics.naze",
   "Task tracker with derived metrics and priority breakdown",
   """-- Task metrics
app "Task Metrics" {
  state tasks = [{title: "Fix bug", priority: "critical", hours: 4}, {title: "Add tests", priority: "high", hours: 8}, {title: "Update docs", priority: "low", hours: 2}]
  computed total = tasks | count
  computed total-hours = tasks | map hours | sum
  computed critical = tasks | filter priority == "critical" | count

  column padding: 20px, gap: 16px {
    heading "Sprint Metrics"
    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #eff6ff, radius: 8px {
        text "Tasks" color: #64748b, font-size: 12px
        text "{total}" font-size: 24px, color: #2563eb
      }
      rect padding: 12px, color: #fef2f2, radius: 8px {
        text "Critical" color: #64748b, font-size: 12px
        text "{critical}" font-size: 24px, color: #ef4444
      }
      rect padding: 12px, color: #f0fdf4, radius: 8px {
        text "Hours" color: #64748b, font-size: 12px
        text "{total-hours}" font-size: 24px, color: #16a34a
      }
    }

    each task in tasks | sort-by priority {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{task.title}" font-weight: bold
        text "{task.priority}" color: #ef4444, font-size: 12px
      }
    }
  }
}""")

ex("gen-pipe-leaderboard-tiers.naze",
   "Leaderboard with tier filtering and top-N display",
   """-- Tiered leaderboard
app "Leaderboard" {
  state players = [{name: "Alice", elo: 2450}, {name: "Bob", elo: 1800}, {name: "Carol", elo: 2100}, {name: "Dan", elo: 1500}]
  computed total = players | count
  computed elite = players | filter elo > 2000 | count

  column padding: 20px, gap: 16px {
    heading "Ranking"
    text "{total} players, {elite} elite" color: #64748b

    each p in players | sort-by elo | take 3 {
      row padding: 8px, color: #fef3c7, radius: 4px, gap: 8px {
        text "{p.name}" font-weight: bold
        text "{p.elo} ELO" color: #92400e
      }
    }
  }
}""")

ex("gen-pipe-order-summary.naze",
   "Order summary with item count and total price",
   """-- Order summary
app "Order" {
  state items = [{name: "Laptop", price: 999, qty: 1}, {name: "Mouse", price: 29, qty: 2}, {name: "Keyboard", price: 79, qty: 1}]
  computed item-count = items | count
  computed total-price = items | map price | sum

  column padding: 20px, gap: 16px {
    heading "Order Summary"
    text "{item-count} items, Subtotal: ${total-price}" color: #2563eb

    each item in items | sort-by price {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 12px {
        text "{item.name}" font-weight: bold
        text "x{item.qty}" color: #64748b
        text "${item.price}" color: #2563eb
      }
    }
  }
}""")

ex("gen-pipe-employee-report.naze",
   "Employee report with department counts and salary totals",
   """-- Employee report
app "HR Report" {
  state employees = [{name: "Alice", dept: "eng", salary: 95000}, {name: "Bob", dept: "design", salary: 85000}, {name: "Carol", dept: "eng", salary: 105000}]
  computed headcount = employees | count
  computed total-salary = employees | map salary | sum
  computed eng-count = employees | filter dept == "eng" | count

  column padding: 20px, gap: 16px {
    heading "HR Report"
    text "{headcount} employees, {eng-count} engineers" color: #64748b
    text "Payroll: ${total-salary}" color: #16a34a

    each emp in employees | sort-by dept {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{emp.name}" font-weight: bold
        text "{emp.dept}" color: #8b5cf6, font-size: 12px
        text "${emp.salary}" color: #16a34a
      }
    }
  }
}""")

ex("gen-pipe-course-catalog.naze",
   "Course catalog with enrollment counts",
   """-- Course catalog
app "Courses" {
  state courses = [{title: "Intro CS", level: "beginner", enrolled: 120}, {title: "ML", level: "advanced", enrolled: 45}, {title: "Web Dev", level: "beginner", enrolled: 200}]
  computed total = courses | count
  computed total-enrolled = courses | map enrolled | sum

  column padding: 20px, gap: 16px {
    heading "Course Catalog"
    text "{total} courses, {total-enrolled} enrolled" color: #64748b

    each course in courses | sort-by enrolled {
      row padding: 12px, color: #f8fafc, radius: 4px, gap: 12px {
        text "{course.title}" font-weight: bold
        text "{course.level}" color: #8b5cf6, font-size: 12px
        text "{course.enrolled} students" color: #64748b
      }
    }
  }
}""")

ex("gen-pipe-product-metrics.naze",
   "Product metrics with review counts and top picks",
   """-- Product metrics
app "Products" {
  state products = [{name: "Widget Pro", rating: 4, reviews: 120}, {name: "Gadget X", rating: 5, reviews: 85}, {name: "Tool Kit", rating: 3, reviews: 200}]
  computed total = products | count
  computed total-reviews = products | map reviews | sum

  column padding: 20px, gap: 16px {
    heading "Product Dashboard"
    text "{total} products, {total-reviews} reviews" color: #f59e0b

    each p in products | sort-by rating | take 2 {
      row padding: 8px, color: #fef3c7, radius: 4px, gap: 8px {
        text "{p.name}" font-weight: bold
        text "{p.rating}/5" color: #f59e0b
      }
    }
  }
}""")

ex("gen-pipe-budget-breakdown.naze",
   "Budget breakdown with income minus expenses",
   """-- Budget breakdown
app "Budget" {
  state expenses = [{label: "Rent", amount: 1500, cat: "housing"}, {label: "Food", amount: 600, cat: "food"}, {label: "Transport", amount: 200, cat: "transport"}]
  state income = 4000
  computed total-spent = expenses | map amount | sum
  computed remaining = income - total-spent

  column padding: 20px, gap: 16px {
    heading "Monthly Budget"
    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #f0fdf4, radius: 8px {
        text "Income" color: #64748b, font-size: 12px
        text "${income}" font-size: 20px, color: #16a34a
      }
      rect padding: 12px, color: #fef2f2, radius: 8px {
        text "Spent" color: #64748b, font-size: 12px
        text "${total-spent}" font-size: 20px, color: #ef4444
      }
      rect padding: 12px, color: #eff6ff, radius: 8px {
        text "Left" color: #64748b, font-size: 12px
        text "${remaining}" font-size: 20px, color: #2563eb
      }
    }

    each exp in expenses | sort-by amount {
      row padding: 4px, gap: 8px {
        text "{exp.label}" font-weight: bold
        text "${exp.amount}" color: #ef4444
      }
    }
  }
}""")

# ─── Category 5: Multi-feature integration ──────────────────────────────────
# 10 examples combining 4+ features

ex("gen-multi-settings.naze",
   "Settings page with storage, input, checkbox, and match",
   """-- Persistent settings
app "Settings" {
  storage username: local "settings-username" default: "user"
  storage theme-pref: local "settings-theme" default: "light"
  state font-size = 16
  state notifications = true

  column padding: 20px, gap: 16px {
    heading "App Settings"
    input bind: username, placeholder: "Display name..."
    text "Hello, {username}!" color: #2563eb

    separator

    match theme-pref {
      "light": text "Light mode active" color: #f59e0b
      "dark": text "Dark mode active" color: #6366f1
      _: text "Default"
    }
    row gap: 8px {
      rect width: 80px, height: 36px, color: #f8fafc, radius: 4px {
        text "Light"
        on click: set theme-pref = "light"
      }
      rect width: 80px, height: 36px, color: #1e293b, radius: 4px {
        text "Dark" color: #ffffff
        on click: set theme-pref = "dark"
      }
    }

    separator

    text "Font Size: {font-size}px" color: #64748b
    row gap: 8px {
      rect width: 40px, height: 32px, color: #e2e8f0, radius: 4px {
        text "A-"
        on click: set font-size = font-size - 2
      }
      rect width: 40px, height: 32px, color: #e2e8f0, radius: 4px {
        text "A+"
        on click: set font-size = font-size + 2
      }
    }

    checkbox bind: notifications, label: "Enable notifications"
    if notifications {
      text "Notifications are ON" color: #16a34a
    }
  }
}""")

ex("gen-multi-todo-storage.naze",
   "Todo list with storage, filters, and computed counts",
   """-- Todo with storage
app "Persistent Todos" {
  storage list-name: local "todo-list-name" default: "My Tasks"
  state task = ""
  state filter = "all"
  state tasks = [{text: "Buy groceries", done: "false"}, {text: "Write report", done: "true"}]
  computed total = tasks | count

  column padding: 20px, gap: 16px {
    heading "{list-name}"
    text "{total} tasks" color: #64748b

    row gap: 8px {
      input bind: task, placeholder: "New task..."
      rect width: 60px, height: 36px, color: #16a34a, radius: 4px {
        text "Add" color: #ffffff
        on click: append task to tasks
      }
    }

    row gap: 8px {
      rect width: 50px, height: 28px, color: #334155, radius: 4px {
        text "All" color: #ffffff, font-size: 12px
        on click: set filter = "all"
      }
      rect width: 60px, height: 28px, color: #16a34a, radius: 4px {
        text "Done" color: #ffffff, font-size: 12px
        on click: set filter = "done"
      }
    }

    each t in tasks {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{t.text}"
      }
    }
  }
}""")

ex("gen-multi-quiz.naze",
   "Quiz with scoring, match display, and computed results",
   """-- Quiz app
app "Quiz" {
  state step = 1
  state score = 0
  state finished = false
  computed percent = score * 50

  column padding: 20px, gap: 16px {
    heading "Knowledge Quiz"
    text "Score: {score}/2 ({percent}%)" color: #2563eb

    if finished == false {
      match step {
        1: column gap: 12px {
          text "Q1: Naze compiles to..." font-weight: bold
          rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
            text "WASM" color: #ffffff
            on click: set score = score + 1
          }
          rect width: 80px, height: 36px, color: #64748b, radius: 4px {
            text "Next" color: #ffffff
            on click: set step = 2
          }
        }
        2: column gap: 12px {
          text "Q2: Naze renders via..." font-weight: bold
          rect width: 100px, height: 36px, color: #2563eb, radius: 4px {
            text "Canvas2D" color: #ffffff
            on click: set score = score + 1
          }
          rect width: 120px, height: 40px, color: #16a34a, radius: 8px {
            text "See Results" color: #ffffff
            on click: set finished = true
          }
        }
        _: text "Complete"
      }
    }

    if finished {
      text "Final Score: {score}/2 ({percent}%)" font-size: 24px, color: #16a34a
    }
  }
}""")

ex("gen-multi-dashboard-full.naze",
   "Full dashboard with state, computed, each, match, input, and grid",
   """-- Full dashboard
app "Analytics" {
  state search = ""
  state view = "overview"
  state visitors = 5200
  state conversions = 340
  computed conv-rate = conversions * 100 / visitors
  state pages = [{url: "/home", views: 2400}, {url: "/about", views: 800}, {url: "/pricing", views: 1200}]
  computed page-count = pages | count

  column padding: 20px, gap: 16px {
    heading "Analytics Dashboard"
    input bind: search, placeholder: "Search pages..."

    row gap: 8px {
      rect width: 100px, height: 32px, color: #2563eb, radius: 4px {
        text "Overview" color: #ffffff
        on click: set view = "overview"
      }
      rect width: 80px, height: 32px, color: #8b5cf6, radius: 4px {
        text "Pages" color: #ffffff
        on click: set view = "pages"
      }
    }

    match view {
      "overview": column gap: 12px {
        grid columns: 2, gap: 12px {
          rect padding: 12px, color: #eff6ff, radius: 8px {
            text "Visitors" color: #64748b, font-size: 12px
            text "{visitors}" font-size: 24px, color: #2563eb
          }
          rect padding: 12px, color: #f0fdf4, radius: 8px {
            text "Conversions" color: #64748b, font-size: 12px
            text "{conversions}" font-size: 24px, color: #16a34a
          }
        }
        text "Rate: {conv-rate}%" color: #2563eb
      }
      "pages": column gap: 8px {
        each page in pages | sort-by views {
          row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
            text "{page.url}" font-weight: bold
            text "{page.views} views" color: #2563eb
          }
        }
      }
      _: text "Unknown"
    }
  }
}""")

ex("gen-multi-profile-editor.naze",
   "Profile editor with storage, select, checkbox, and match mode",
   """-- Profile editor
app "My Profile" {
  storage display-name: local "profile-name" default: "Anonymous"
  state email = ""
  state visibility = "public"
  state show-email = true
  state mode = "view"

  column padding: 20px, gap: 16px {
    heading "Profile"
    row gap: 8px {
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "View" color: #ffffff
        on click: set mode = "view"
      }
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "Edit" color: #ffffff
        on click: set mode = "edit"
      }
    }

    match mode {
      "view": column gap: 8px {
        text "Name: {display-name}" font-size: 20px
        text "Visibility: {visibility}" color: #94a3b8
        if show-email {
          text "Email is public" color: #16a34a
        }
      }
      "edit": column gap: 12px {
        input bind: display-name, placeholder: "Display name"
        input bind: email, placeholder: "Email"
        select bind: visibility {
          option "Public" value: "public"
          option "Private" value: "private"
        }
        checkbox bind: show-email, label: "Show email publicly"
      }
      _: text "Error"
    }
  }
}""")

ex("gen-multi-recipe-planner.naze",
   "Recipe planner with storage, each, match, and input",
   """-- Recipe planner
app "Meal Planner" {
  storage week-name: local "planner-week" default: "This Week"
  state day = "monday"
  state meals = [{name: "Pasta", day: "monday"}, {name: "Salad", day: "tuesday"}, {name: "Tacos", day: "wednesday"}]
  computed meal-count = meals | count

  column padding: 20px, gap: 16px {
    heading "{week-name}"
    text "{meal-count} meals planned" color: #64748b

    match day {
      "monday": text "Monday selected" color: #2563eb
      "tuesday": text "Tuesday selected" color: #2563eb
      _: text "Select a day"
    }

    row gap: 4px {
      rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
        text "M" color: #ffffff, font-size: 12px
        on click: set day = "monday"
      }
      rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
        text "T" color: #ffffff, font-size: 12px
        on click: set day = "tuesday"
      }
      rect width: 40px, height: 28px, color: #2563eb, radius: 4px {
        text "W" color: #ffffff, font-size: 12px
        on click: set day = "wednesday"
      }
    }

    each meal in meals | sort-by day {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
        text "{meal.day}" font-weight: bold, color: #2563eb
        text "{meal.name}" color: #374151
      }
    }
  }
}""")

ex("gen-multi-kanban-full.naze",
   "Kanban board with input, storage, computed, and each",
   """-- Full Kanban
app "Kanban Pro" {
  storage board-name: local "kanban-name" default: "Sprint Board"
  state new-task = ""
  state todo = [{title: "Design UI", tag: "design"}, {title: "Write tests", tag: "dev"}]
  state doing = [{title: "Build API", tag: "dev"}]
  state done = [{title: "Setup repo", tag: "ops"}]
  computed todo-count = todo | count
  computed done-count = done | count

  column padding: 20px, gap: 16px {
    heading "{board-name}"
    row gap: 8px {
      input bind: new-task, placeholder: "New task..."
      rect width: 60px, height: 36px, color: #16a34a, radius: 4px {
        text "Add" color: #ffffff
        on click: append new-task to todo
      }
    }

    row gap: 16px {
      column gap: 8px, padding: 12px, color: #fef2f2, radius: 8px {
        text "Todo ({todo-count})" font-weight: bold, color: #dc2626
        each item in todo {
          rect padding: 8px, color: #ffffff, radius: 4px {
            text "{item.title}"
          }
        }
      }
      column gap: 8px, padding: 12px, color: #f0fdf4, radius: 8px {
        text "Done ({done-count})" font-weight: bold, color: #16a34a
        each item in done {
          rect padding: 8px, color: #ffffff, radius: 4px {
            text "{item.title}"
          }
        }
      }
    }
  }
}""")

ex("gen-multi-notes-app.naze",
   "Notes app with storage, each, match, and computed",
   """-- Notes with storage
app "Notes Pro" {
  storage author: local "notes-author" default: "Me"
  state view = "list"
  state notes = [{title: "Meeting Notes", body: "Discuss roadmap", tag: "work"}, {title: "Shopping", body: "Buy milk", tag: "personal"}]
  computed note-count = notes | count

  column padding: 20px, gap: 16px {
    heading "Notes by {author}"
    text "{note-count} notes" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "List" color: #ffffff
        on click: set view = "list"
      }
      rect width: 60px, height: 32px, color: #8b5cf6, radius: 4px {
        text "Add" color: #ffffff
        on click: set view = "add"
      }
    }

    if view == "add" {
      text "Add note form" color: #64748b
    }

    if view == "list" {
      each note in notes | sort-by tag {
        row padding: 12px, color: #f8fafc, radius: 8px, gap: 8px {
          column gap: 2px {
            text "{note.title}" font-weight: bold
            text "{note.tag}" color: #8b5cf6, font-size: 10px
          }
        }
      }
    }
  }
}""")

ex("gen-multi-fitness-tracker.naze",
   "Fitness tracker with timer, computed, each, match, and storage",
   """-- Fitness tracker
app "FitLog" {
  storage username: local "fit-username" default: "Athlete"
  state elapsed = 0
  state tab = "workout"
  state exercises = [{name: "Running", cal: 300, mins: 30}, {name: "Cycling", cal: 200, mins: 20}]
  computed total-cal = exercises | map cal | sum

  timer session-timer: every 1s {
    set elapsed = elapsed + 1
  }

  column padding: 20px, gap: 16px {
    heading "FitLog - {username}"
    text "{elapsed}s elapsed" color: #2563eb

    row gap: 8px {
      rect width: 80px, height: 32px, color: #16a34a, radius: 4px {
        text "Workout" color: #ffffff
        on click: set tab = "workout"
      }
      rect width: 80px, height: 32px, color: #f59e0b, radius: 4px {
        text "Stats" color: #ffffff
        on click: set tab = "stats"
      }
    }

    if tab == "stats" {
      text "Total calories: {total-cal}" font-size: 24px, color: #16a34a
    }

    if tab == "workout" {
      each ex in exercises {
        row padding: 12px, color: #f0fdf4, radius: 8px, gap: 8px {
          text "{ex.name}" font-weight: bold
          text "{ex.cal} cal" color: #16a34a
        }
      }
    }
  }
}""")

ex("gen-multi-expense-full.naze",
   "Expense manager with storage, computed, match, and each",
   """-- Full expense manager
app "Expense Manager" {
  storage account-name: local "expense-acct" default: "Personal"
  state view = "list"
  state expenses = [{desc: "Coffee", amount: "5", cat: "food"}, {desc: "Bus pass", amount: "80", cat: "transport"}, {desc: "Gym", amount: "40", cat: "health"}]
  computed expense-count = expenses | count
  computed total = expenses | map amount | sum

  column padding: 20px, gap: 16px {
    heading "{account-name} Expenses"
    text "{expense-count} items, Total: ${total}" color: #ef4444

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "List" color: #ffffff
        on click: set view = "list"
      }
      rect width: 60px, height: 32px, color: #16a34a, radius: 4px {
        text "Add" color: #ffffff
        on click: set view = "add"
      }
    }

    if view == "add" {
      text "Add expense form here" color: #64748b
    }

    if view == "list" {
      each exp in expenses | sort-by cat {
        row padding: 8px, color: #f8fafc, radius: 4px, gap: 8px {
          text "{exp.desc}" font-weight: bold
          text "${exp.amount}" color: #ef4444
          text "{exp.cat}" color: #94a3b8, font-size: 10px
        }
      }
    }
  }
}""")

# ─── Category 6: Grid and responsive patterns ───────────────────────────────
# 10 examples using grid layout with various configurations

ex("gen-grid-2col-cards.naze",
   "Two-column card grid with alternating colors",
   """-- Two-column cards
app "Card Grid 2x" {
  column padding: 20px, gap: 16px {
    heading "Card Grid"
    grid columns: 2, gap: 16px {
      rect padding: 16px, color: #eff6ff, radius: 8px {
        text "Card 1" font-weight: bold
        text "First card content" color: #64748b
      }
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "Card 2" font-weight: bold
        text "Second card content" color: #64748b
      }
      rect padding: 16px, color: #fef3c7, radius: 8px {
        text "Card 3" font-weight: bold
      }
      rect padding: 16px, color: #fce7f3, radius: 8px {
        text "Card 4" font-weight: bold
      }
    }
  }
}""")

ex("gen-grid-3col-icons.naze",
   "Three-column grid of icon-like squares with labels",
   """-- Icon grid
app "Icon Grid" {
  column padding: 20px, gap: 16px {
    heading "Icon Grid"
    grid columns: 3, gap: 12px {
      column gap: 4px {
        rect width: 60px, height: 60px, color: #ef4444, radius: 12px
        text "Home" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 60px, height: 60px, color: #f59e0b, radius: 12px
        text "Search" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 60px, height: 60px, color: #22c55e, radius: 12px
        text "Profile" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 60px, height: 60px, color: #3b82f6, radius: 12px
        text "Settings" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 60px, height: 60px, color: #8b5cf6, radius: 12px
        text "Help" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 60px, height: 60px, color: #ec4899, radius: 12px
        text "Logout" color: #64748b, font-size: 12px
      }
    }
  }
}""")

ex("gen-grid-4col-metrics.naze",
   "Four-column metrics grid with stat cards",
   """-- Four-column metrics
app "Metrics 4x" {
  state a = 1250
  state b = 340
  state c = 89
  state d = 42

  column padding: 20px, gap: 16px {
    heading "Key Metrics"
    grid columns: 4, gap: 12px {
      rect padding: 12px, color: #eff6ff, radius: 8px {
        text "Users" color: #64748b, font-size: 11px
        text "{a}" font-size: 20px, color: #2563eb
      }
      rect padding: 12px, color: #f0fdf4, radius: 8px {
        text "Sales" color: #64748b, font-size: 11px
        text "{b}" font-size: 20px, color: #16a34a
      }
      rect padding: 12px, color: #fef3c7, radius: 8px {
        text "Tasks" color: #64748b, font-size: 11px
        text "{c}" font-size: 20px, color: #f59e0b
      }
      rect padding: 12px, color: #fce7f3, radius: 8px {
        text "Errors" color: #64748b, font-size: 11px
        text "{d}" font-size: 20px, color: #ef4444
      }
    }
  }
}""")

ex("gen-grid-nested-row.naze",
   "Grid with rows nested inside grid cells",
   """-- Grid with nested rows
app "Nested Grid" {
  column padding: 20px, gap: 16px {
    heading "Complex Layout"
    grid columns: 2, gap: 16px {
      rect padding: 16px, color: #f8fafc, radius: 8px {
        column gap: 8px {
          text "Card Header" font-weight: bold
          row gap: 8px {
            rect width: 40px, height: 40px, color: #3b82f6, radius: 8px
            column gap: 2px {
              text "Title" font-weight: bold
              text "Subtitle" color: #64748b, font-size: 12px
            }
          }
        }
      }
      rect padding: 16px, color: #f8fafc, radius: 8px {
        column gap: 8px {
          text "Stats Card" font-weight: bold
          row gap: 16px {
            text "Views: 1234" color: #64748b
            text "Likes: 567" color: #64748b
          }
        }
      }
    }
  }
}""")

ex("gen-grid-container-wrap.naze",
   "Grid inside a container with max-width",
   """-- Contained grid
app "Centered Grid" {
  container max-width: 800px, padding: 20px {
    column gap: 16px {
      heading "Contained Grid Layout"
      grid columns: 3, gap: 12px {
        rect padding: 16px, color: #eff6ff, radius: 8px {
          text "Feature 1" font-weight: bold
        }
        rect padding: 16px, color: #f0fdf4, radius: 8px {
          text "Feature 2" font-weight: bold
        }
        rect padding: 16px, color: #fef3c7, radius: 8px {
          text "Feature 3" font-weight: bold
        }
      }
    }
  }
}""")

ex("gen-grid-mixed-layout.naze",
   "Page with header row, grid body, and footer",
   """-- Mixed layout
app "Mixed Layout" {
  column gap: 0px {
    row padding: 16px, color: #1e293b, gap: 16px {
      heading "SiteTitle" color: #ffffff, font-size: 18px
      text "Home" color: #94a3b8
      text "About" color: #94a3b8
    }

    column padding: 20px, gap: 16px {
      heading "Featured"
      grid columns: 3, gap: 12px {
        rect padding: 16px, color: #eff6ff, radius: 8px {
          text "Article 1" font-weight: bold
        }
        rect padding: 16px, color: #f0fdf4, radius: 8px {
          text "Article 2" font-weight: bold
        }
        rect padding: 16px, color: #fef3c7, radius: 8px {
          text "Article 3" font-weight: bold
        }
      }
    }

    row padding: 16px, color: #f1f5f9 {
      text "Footer content" color: #64748b
    }
  }
}""")

ex("gen-grid-sidebar-main.naze",
   "Sidebar navigation with main grid content",
   """-- Sidebar + main grid
app "Sidebar Layout" {
  state section = "home"

  row gap: 0px {
    column width: 200px, padding: 16px, gap: 8px, color: #f8fafc {
      text "Navigation" font-weight: bold
      rect padding: 8px, color: #eff6ff, radius: 4px {
        text "Home"
        on click: set section = "home"
      }
      rect padding: 8px, color: #eff6ff, radius: 4px {
        text "Products"
        on click: set section = "products"
      }
    }

    column padding: 20px, gap: 16px {
      heading "Content Area"
      grid columns: 2, gap: 12px {
        rect padding: 16px, color: #f8fafc, radius: 8px {
          text "Panel 1" font-weight: bold
        }
        rect padding: 16px, color: #f8fafc, radius: 8px {
          text "Panel 2" font-weight: bold
        }
      }
    }
  }
}""")

ex("gen-grid-gallery.naze",
   "Photo gallery with varying column counts",
   """-- Photo gallery grid
app "Photo Gallery" {
  state section = "featured"

  column padding: 20px, gap: 16px {
    heading "Photo Gallery"
    row gap: 8px {
      rect width: 100px, height: 32px, color: #2563eb, radius: 4px {
        text "Featured" color: #ffffff
        on click: set section = "featured"
      }
      rect width: 80px, height: 32px, color: #64748b, radius: 4px {
        text "All" color: #ffffff
        on click: set section = "all"
      }
    }

    match section {
      "featured": grid columns: 2, gap: 8px {
        rect width: 150px, height: 120px, color: #dbeafe, radius: 8px
        rect width: 150px, height: 120px, color: #dcfce7, radius: 8px
      }
      "all": grid columns: 3, gap: 8px {
        rect width: 100px, height: 80px, color: #dbeafe, radius: 8px
        rect width: 100px, height: 80px, color: #dcfce7, radius: 8px
        rect width: 100px, height: 80px, color: #fef3c7, radius: 8px
        rect width: 100px, height: 80px, color: #fce7f3, radius: 8px
        rect width: 100px, height: 80px, color: #e0e7ff, radius: 8px
        rect width: 100px, height: 80px, color: #f3e8ff, radius: 8px
      }
      _: text "Unknown"
    }
  }
}""")

ex("gen-grid-pricing-3.naze",
   "Three-column pricing grid with tiered plans",
   """-- Pricing grid
app "Plans" {
  state selected = "none"

  column padding: 20px, gap: 16px {
    heading "Choose Your Plan"
    grid columns: 3, gap: 16px {
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Starter" font-weight: bold
          text "$9/mo" font-size: 24px, color: #2563eb
          separator
          text "5 projects" color: #64748b
          rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
            text "Select" color: #ffffff
            on click: set selected = "starter"
          }
        }
      }
      rect padding: 20px, color: #eff6ff, radius: 12px {
        column gap: 8px {
          text "Pro" font-weight: bold
          text "$29/mo" font-size: 24px, color: #2563eb
          separator
          text "Unlimited" color: #64748b
          rect width: 100px, height: 36px, color: #2563eb, radius: 8px {
            text "Select" color: #ffffff
            on click: set selected = "pro"
          }
        }
      }
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Enterprise" font-weight: bold
          text "Custom" font-size: 24px, color: #64748b
          separator
          text "SLA guarantee" color: #64748b
          rect width: 100px, height: 36px, color: #64748b, radius: 8px {
            text "Contact" color: #ffffff
            on click: set selected = "enterprise"
          }
        }
      }
    }

    if selected != "none" {
      text "Selected: {selected}" color: #16a34a, font-size: 18px
    }
  }
}""")

ex("gen-grid-responsive-dashboard.naze",
   "Dashboard with 2-col metrics, 3-col features, and footer",
   """-- Responsive dashboard grid
app "Dashboard Grid" {
  state users = 5400
  state revenue = 128000

  column padding: 20px, gap: 16px {
    heading "Business Dashboard"
    grid columns: 2, gap: 16px {
      rect padding: 20px, color: #eff6ff, radius: 12px {
        column gap: 4px {
          text "Total Users" color: #64748b
          text "{users}" font-size: 32px, color: #2563eb
        }
      }
      rect padding: 20px, color: #f0fdf4, radius: 12px {
        column gap: 4px {
          text "Revenue" color: #64748b
          text "${revenue}" font-size: 32px, color: #16a34a
        }
      }
    }

    separator

    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #faf5ff, radius: 8px {
        text "Analytics" font-weight: bold
      }
      rect padding: 12px, color: #fff7ed, radius: 8px {
        text "Reports" font-weight: bold
      }
      rect padding: 12px, color: #ecfeff, radius: 8px {
        text "Alerts" font-weight: bold
      }
    }

    row padding: 12px, color: #f1f5f9, radius: 4px {
      text "Dashboard v2.1" color: #94a3b8, font-size: 12px
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# GENERATORS: Additional domain patterns (75 examples)
# ═══════════════════════════════════════════════════════════════════════════════

# ─── Stat cards (12) ─────────────────────────────────────────────────────────

STAT_T = """-- __DESC__
app "__TITLE__" {
  state __A__ = __VA__
  state __B__ = __VB__

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    grid columns: 2, gap: 12px {
      rect padding: 16px, color: __BG1__, radius: 8px {
        text "__LA__" color: #64748b, font-size: 12px
        text "{__A__}" font-size: 24px, color: __CLR1__
      }
      rect padding: 16px, color: __BG2__, radius: 8px {
        text "__LB__" color: #64748b, font-size: 12px
        text "{__B__}" font-size: 24px, color: __CLR2__
      }
    }
    row gap: 8px {
      rect width: 80px, height: 32px, color: __CLR1__, radius: 4px {
        text "+1" color: #ffffff
        on click: set __A__ = __A__ + 1
      }
      rect width: 80px, height: 32px, color: #ef4444, radius: 4px {
        text "Reset" color: #ffffff
        on click: set __A__ = __VA__
      }
    }
  }
}"""

for n, cfg in [
    ("visitors", {"TITLE": "Site Stats", "DESC": "Website visitor statistics", "A": "visitors", "VA": "1420", "B": "bounces", "VB": "310", "LA": "Visitors", "LB": "Bounces", "BG1": "#eff6ff", "BG2": "#fef2f2", "CLR1": "#2563eb", "CLR2": "#ef4444"}),
    ("orders", {"TITLE": "Order Stats", "DESC": "E-commerce order metrics", "A": "orders", "VA": "87", "B": "returns", "VB": "12", "LA": "Orders", "LB": "Returns", "BG1": "#f0fdf4", "BG2": "#fff7ed", "CLR1": "#16a34a", "CLR2": "#f97316"}),
    ("tickets", {"TITLE": "Support", "DESC": "Support ticket overview", "A": "open-tickets", "VA": "23", "B": "resolved", "VB": "145", "LA": "Open", "LB": "Resolved", "BG1": "#fef3c7", "BG2": "#ecfdf5", "CLR1": "#f59e0b", "CLR2": "#10b981"}),
    ("students", {"TITLE": "Class Stats", "DESC": "Classroom attendance tracker", "A": "present", "VA": "28", "B": "absent", "VB": "4", "LA": "Present", "LB": "Absent", "BG1": "#eff6ff", "BG2": "#fce7f3", "CLR1": "#3b82f6", "CLR2": "#ec4899"}),
    ("commits", {"TITLE": "Repo Stats", "DESC": "Git repository statistics", "A": "commits", "VA": "342", "B": "issues", "VB": "17", "LA": "Commits", "LB": "Issues", "BG1": "#f5f3ff", "BG2": "#fef2f2", "CLR1": "#8b5cf6", "CLR2": "#dc2626"}),
    ("downloads", {"TITLE": "Package Stats", "DESC": "Package download metrics", "A": "downloads", "VA": "9800", "B": "stars", "VB": "240", "LA": "Downloads", "LB": "Stars", "BG1": "#ecfeff", "BG2": "#fef3c7", "CLR1": "#06b6d4", "CLR2": "#eab308"}),
    ("members", {"TITLE": "Team Stats", "DESC": "Team member statistics", "A": "members", "VA": "15", "B": "projects", "VB": "8", "LA": "Members", "LB": "Projects", "BG1": "#f0fdf4", "BG2": "#eff6ff", "CLR1": "#22c55e", "CLR2": "#3b82f6"}),
    ("patients", {"TITLE": "Clinic Stats", "DESC": "Clinic patient statistics", "A": "patients", "VA": "45", "B": "appointments", "VB": "12", "LA": "Patients", "LB": "Appts Today", "BG1": "#faf5ff", "BG2": "#ecfdf5", "CLR1": "#a855f7", "CLR2": "#059669"}),
    ("messages", {"TITLE": "Inbox Stats", "DESC": "Email inbox statistics", "A": "unread", "VA": "18", "B": "sent", "VB": "56", "LA": "Unread", "LB": "Sent", "BG1": "#fef2f2", "BG2": "#f0f9ff", "CLR1": "#ef4444", "CLR2": "#0284c7"}),
    ("inventory", {"TITLE": "Stock Stats", "DESC": "Inventory stock levels", "A": "in-stock", "VA": "340", "B": "low-stock", "VB": "12", "LA": "In Stock", "LB": "Low Stock", "BG1": "#f0fdf4", "BG2": "#fef3c7", "CLR1": "#16a34a", "CLR2": "#d97706"}),
    ("workouts", {"TITLE": "Gym Stats", "DESC": "Workout session tracker", "A": "sessions", "VA": "48", "B": "streak", "VB": "7", "LA": "Sessions", "LB": "Streak", "BG1": "#eff6ff", "BG2": "#f0fdf4", "CLR1": "#2563eb", "CLR2": "#16a34a"}),
    ("meals", {"TITLE": "Diet Stats", "DESC": "Daily meal tracking", "A": "calories", "VA": "1850", "B": "meals", "VB": "4", "LA": "Calories", "LB": "Meals", "BG1": "#fff7ed", "BG2": "#faf5ff", "CLR1": "#ea580c", "CLR2": "#7c3aed"}),
]:
    ex(f"gen-stat-{n}.naze", cfg["DESC"], fill(STAT_T, cfg))

# ─── Panels with toggle (10) ────────────────────────────────────────────────

PANEL_T = """-- __DESC__
app "__TITLE__" {
  state expanded = false

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    rect padding: 16px, color: #f8fafc, radius: 8px {
      column gap: 8px {
        row gap: 8px {
          text "__HEADING__" font-weight: bold
          rect width: 70px, height: 28px, color: __CLR__, radius: 4px {
            text "Toggle" color: #ffffff, font-size: 12px
            on click: set expanded = true
          }
        }

        if expanded {
          separator
          text "__LINE1__" color: #334155
          text "__LINE2__" color: #64748b
          text "__LINE3__" color: #94a3b8, font-size: 12px
        }
      }
    }
  }
}"""

for n, cfg in [
    ("faq1", {"TITLE": "FAQ", "DESC": "FAQ with expandable answer", "HEADING": "What is Naze?", "LINE1": "Naze is a declarative UI language.", "LINE2": "It compiles to WASM and renders via Canvas2D.", "LINE3": "No HTML, no CSS, no JavaScript.", "CLR": "#2563eb"}),
    ("faq2", {"TITLE": "FAQ Pricing", "DESC": "Pricing FAQ with toggle", "HEADING": "Is it free?", "LINE1": "Yes, Naze is open source and free.", "LINE2": "MIT licensed for personal and commercial use.", "LINE3": "Community support available on GitHub.", "CLR": "#16a34a"}),
    ("bio", {"TITLE": "Bio Panel", "DESC": "Expandable biography panel", "HEADING": "About the Author", "LINE1": "Software engineer with 10 years experience.", "LINE2": "Specializes in language design and compilers.", "LINE3": "Based in San Francisco, CA.", "CLR": "#8b5cf6"}),
    ("specs", {"TITLE": "Specs Panel", "DESC": "Product specifications toggle", "HEADING": "Technical Specifications", "LINE1": "Dimensions: 150mm x 75mm x 8mm", "LINE2": "Weight: 185g, Material: Aluminum", "LINE3": "Battery: 4500mAh, USB-C charging", "CLR": "#f59e0b"}),
    ("terms", {"TITLE": "Terms Panel", "DESC": "Terms of service toggle", "HEADING": "Terms of Service", "LINE1": "By using this service you agree to our terms.", "LINE2": "Data is processed securely and never shared.", "LINE3": "Last updated: January 2026", "CLR": "#64748b"}),
    ("recipe-info", {"TITLE": "Recipe Info", "DESC": "Recipe details toggle", "HEADING": "Cooking Instructions", "LINE1": "Preheat oven to 180C (350F).", "LINE2": "Mix ingredients and bake for 25 minutes.", "LINE3": "Serves 4 people. Prep time: 15 min.", "CLR": "#ef4444"}),
    ("changelog-info", {"TITLE": "Changelog", "DESC": "Version changelog toggle", "HEADING": "Version 2.1 Changes", "LINE1": "Added dark mode support.", "LINE2": "Fixed navigation bug on mobile.", "LINE3": "Performance improvements: 30% faster rendering.", "CLR": "#06b6d4"}),
    ("shipping", {"TITLE": "Shipping Info", "DESC": "Shipping details toggle", "HEADING": "Shipping Information", "LINE1": "Free shipping on orders over $50.", "LINE2": "Standard delivery: 3-5 business days.", "LINE3": "Express delivery available for $9.99.", "CLR": "#ec4899"}),
    ("privacy", {"TITLE": "Privacy Panel", "DESC": "Privacy policy toggle", "HEADING": "Privacy Policy", "LINE1": "We collect minimal data for service operation.", "LINE2": "No data is sold to third parties.", "LINE3": "You can request data deletion anytime.", "CLR": "#14b8a6"}),
    ("warranty", {"TITLE": "Warranty Panel", "DESC": "Warranty information toggle", "HEADING": "Warranty Coverage", "LINE1": "2-year limited warranty included.", "LINE2": "Covers manufacturing defects only.", "LINE3": "Contact support for warranty claims.", "CLR": "#6366f1"}),
]:
    ex(f"gen-panel-{n}.naze", cfg["DESC"], fill(PANEL_T, cfg))

# ─── Select + match combos (10) ─────────────────────────────────────────────

SELECT_T = """-- __DESC__
app "__TITLE__" {
  state choice = "__DEFAULT__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    select bind: choice {
      option "__OPT1__" value: "__V1__"
      option "__OPT2__" value: "__V2__"
      option "__OPT3__" value: "__V3__"
    }

    match choice {
      "__V1__": rect padding: 16px, color: __BG1__, radius: 8px {
        text "__MSG1__" color: __CLR1__
      }
      "__V2__": rect padding: 16px, color: __BG2__, radius: 8px {
        text "__MSG2__" color: __CLR2__
      }
      "__V3__": rect padding: 16px, color: __BG3__, radius: 8px {
        text "__MSG3__" color: __CLR3__
      }
      _: text "Select an option" color: #94a3b8
    }
  }
}"""

for n, cfg in [
    ("language", {"TITLE": "Language", "DESC": "Language selector with preview", "DEFAULT": "en", "OPT1": "English", "V1": "en", "OPT2": "Spanish", "V2": "es", "OPT3": "French", "V3": "fr", "BG1": "#eff6ff", "CLR1": "#2563eb", "MSG1": "Hello! Welcome.", "BG2": "#fef3c7", "CLR2": "#d97706", "MSG2": "Hola! Bienvenido.", "BG3": "#fce7f3", "CLR3": "#be185d", "MSG3": "Bonjour! Bienvenue."}),
    ("size", {"TITLE": "Size Picker", "DESC": "Product size selector", "DEFAULT": "medium", "OPT1": "Small", "V1": "small", "OPT2": "Medium", "V2": "medium", "OPT3": "Large", "V3": "large", "BG1": "#f0fdf4", "CLR1": "#16a34a", "MSG1": "Small: fits 5-7 oz", "BG2": "#eff6ff", "CLR2": "#2563eb", "MSG2": "Medium: fits 8-12 oz", "BG3": "#faf5ff", "CLR3": "#7c3aed", "MSG3": "Large: fits 14-20 oz"}),
    ("priority", {"TITLE": "Priority", "DESC": "Task priority selector", "DEFAULT": "medium", "OPT1": "Low", "V1": "low", "OPT2": "Medium", "V2": "medium", "OPT3": "High", "V3": "high", "BG1": "#f0fdf4", "CLR1": "#16a34a", "MSG1": "Low priority: handle when free", "BG2": "#fef3c7", "CLR2": "#d97706", "MSG2": "Medium: address this week", "BG3": "#fef2f2", "CLR3": "#dc2626", "MSG3": "High priority: handle immediately"}),
    ("theme-sel", {"TITLE": "Theme", "DESC": "Theme color selector", "DEFAULT": "light", "OPT1": "Light", "V1": "light", "OPT2": "Dark", "V2": "dark", "OPT3": "System", "V3": "system", "BG1": "#f8fafc", "CLR1": "#334155", "MSG1": "Light mode active", "BG2": "#1e293b", "CLR2": "#e2e8f0", "MSG2": "Dark mode active", "BG3": "#f1f5f9", "CLR3": "#64748b", "MSG3": "Following system preference"}),
    ("region", {"TITLE": "Region", "DESC": "Region selector for content", "DEFAULT": "us", "OPT1": "US", "V1": "us", "OPT2": "EU", "V2": "eu", "OPT3": "Asia", "V3": "asia", "BG1": "#eff6ff", "CLR1": "#1d4ed8", "MSG1": "United States region selected", "BG2": "#ecfdf5", "CLR2": "#059669", "MSG2": "European Union region selected", "BG3": "#fef2f2", "CLR3": "#dc2626", "MSG3": "Asia Pacific region selected"}),
    ("plan", {"TITLE": "Plan Selector", "DESC": "Subscription plan chooser", "DEFAULT": "basic", "OPT1": "Basic", "V1": "basic", "OPT2": "Pro", "V2": "pro", "OPT3": "Enterprise", "V3": "enterprise", "BG1": "#f8fafc", "CLR1": "#64748b", "MSG1": "Basic: 5 GB storage, email support", "BG2": "#eff6ff", "CLR2": "#2563eb", "MSG2": "Pro: 50 GB storage, priority support", "BG3": "#faf5ff", "CLR3": "#7c3aed", "MSG3": "Enterprise: unlimited, dedicated support"}),
    ("currency", {"TITLE": "Currency", "DESC": "Currency display selector", "DEFAULT": "usd", "OPT1": "USD", "V1": "usd", "OPT2": "EUR", "V2": "eur", "OPT3": "GBP", "V3": "gbp", "BG1": "#ecfdf5", "CLR1": "#059669", "MSG1": "Prices shown in US Dollars", "BG2": "#eff6ff", "CLR2": "#2563eb", "MSG2": "Prices shown in Euros", "BG3": "#fef3c7", "CLR3": "#d97706", "MSG3": "Prices shown in British Pounds"}),
    ("difficulty", {"TITLE": "Difficulty", "DESC": "Game difficulty selector", "DEFAULT": "normal", "OPT1": "Easy", "V1": "easy", "OPT2": "Normal", "V2": "normal", "OPT3": "Hard", "V3": "hard", "BG1": "#f0fdf4", "CLR1": "#16a34a", "MSG1": "Easy: relaxed gameplay, extra lives", "BG2": "#fef3c7", "CLR2": "#d97706", "MSG2": "Normal: balanced challenge", "BG3": "#fef2f2", "CLR3": "#dc2626", "MSG3": "Hard: limited lives, no checkpoints"}),
    ("font-size", {"TITLE": "Font Size", "DESC": "Accessibility font size selector", "DEFAULT": "medium", "OPT1": "Small", "V1": "small", "OPT2": "Medium", "V2": "medium", "OPT3": "Large", "V3": "large", "BG1": "#f8fafc", "CLR1": "#64748b", "MSG1": "Small text: compact display", "BG2": "#eff6ff", "CLR2": "#2563eb", "MSG2": "Medium text: default reading size", "BG3": "#faf5ff", "CLR3": "#7c3aed", "MSG3": "Large text: enhanced readability"}),
    ("sort-by", {"TITLE": "Sort Order", "DESC": "Data sort order selector", "DEFAULT": "name", "OPT1": "By Name", "V1": "name", "OPT2": "By Date", "V2": "date", "OPT3": "By Rating", "V3": "rating", "BG1": "#eff6ff", "CLR1": "#2563eb", "MSG1": "Sorted alphabetically by name", "BG2": "#ecfdf5", "CLR2": "#059669", "MSG2": "Sorted by most recent first", "BG3": "#fef3c7", "CLR3": "#d97706", "MSG3": "Sorted by highest rating first"}),
]:
    ex(f"gen-sel-{n}.naze", cfg["DESC"], fill(SELECT_T, cfg))

# ─── Input + display combos (10) ────────────────────────────────────────────

INPUT_T = """-- __DESC__
app "__TITLE__" {
  state __VAR__ = ""
  state submitted = false

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__PROMPT__" color: #64748b

    input bind: __VAR__, placeholder: "__PLACEHOLDER__"

    row gap: 8px {
      rect width: 100px, height: 36px, color: __CLR__, radius: 4px {
        text "__BTN__" color: #ffffff
        on click: set submitted = true
      }
      rect width: 80px, height: 36px, color: #94a3b8, radius: 4px {
        text "Clear" color: #ffffff
        on click: set __VAR__ = ""
        on click: set submitted = false
      }
    }

    if submitted {
      rect padding: 12px, color: __BG__, radius: 8px {
        text "__RESULT_PREFIX__ {__VAR__}" font-weight: bold, color: __CLR__
      }
    }
  }
}"""

for n, cfg in [
    ("username", {"TITLE": "Username", "DESC": "Username input with preview", "VAR": "username", "PROMPT": "Choose a username", "PLACEHOLDER": "Enter username", "BTN": "Submit", "CLR": "#2563eb", "BG": "#eff6ff", "RESULT_PREFIX": "Welcome,"}),
    ("email-sub", {"TITLE": "Newsletter", "DESC": "Email subscription form", "VAR": "email", "PROMPT": "Subscribe to our newsletter", "PLACEHOLDER": "your@email.com", "BTN": "Subscribe", "CLR": "#16a34a", "BG": "#f0fdf4", "RESULT_PREFIX": "Subscribed:"}),
    ("search-q", {"TITLE": "Search", "DESC": "Search with results display", "VAR": "query", "PROMPT": "Search our catalog", "PLACEHOLDER": "Search...", "BTN": "Search", "CLR": "#8b5cf6", "BG": "#faf5ff", "RESULT_PREFIX": "Results for:"}),
    ("feedback", {"TITLE": "Feedback", "DESC": "Feedback submission form", "VAR": "message", "PROMPT": "Share your feedback", "PLACEHOLDER": "Type your feedback...", "BTN": "Send", "CLR": "#f59e0b", "BG": "#fef3c7", "RESULT_PREFIX": "Thanks for:"}),
    ("coupon", {"TITLE": "Coupon", "DESC": "Coupon code entry form", "VAR": "code", "PROMPT": "Enter your coupon code", "PLACEHOLDER": "SAVE20", "BTN": "Apply", "CLR": "#ef4444", "BG": "#fef2f2", "RESULT_PREFIX": "Applied:"}),
    ("nickname", {"TITLE": "Nickname", "DESC": "Display name editor", "VAR": "nickname", "PROMPT": "Set your display name", "PLACEHOLDER": "Enter nickname", "BTN": "Save", "CLR": "#06b6d4", "BG": "#ecfeff", "RESULT_PREFIX": "Name set to:"}),
    ("invite", {"TITLE": "Invite", "DESC": "Team invitation form", "VAR": "invite-email", "PROMPT": "Invite a team member", "PLACEHOLDER": "colleague@work.com", "BTN": "Invite", "CLR": "#ec4899", "BG": "#fce7f3", "RESULT_PREFIX": "Invited:"}),
    ("api-key", {"TITLE": "API Key", "DESC": "API key configuration", "VAR": "api-key", "PROMPT": "Enter your API key", "PLACEHOLDER": "sk-...", "BTN": "Save Key", "CLR": "#1e293b", "BG": "#f1f5f9", "RESULT_PREFIX": "Key saved:"}),
    ("tag", {"TITLE": "Add Tag", "DESC": "Tag input for categorization", "VAR": "tag-name", "PROMPT": "Add a tag to this item", "PLACEHOLDER": "e.g. important", "BTN": "Add Tag", "CLR": "#7c3aed", "BG": "#f5f3ff", "RESULT_PREFIX": "Tagged:"}),
    ("rename", {"TITLE": "Rename", "DESC": "File rename dialog", "VAR": "new-name", "PROMPT": "Enter new name for this file", "PLACEHOLDER": "document.txt", "BTN": "Rename", "CLR": "#0891b2", "BG": "#ecfeff", "RESULT_PREFIX": "Renamed to:"}),
]:
    ex(f"gen-input-{n}.naze", cfg["DESC"], fill(INPUT_T, cfg))

# ─── Checkbox config pages (10) ─────────────────────────────────────────────

CHECK_T = """-- __DESC__
app "__TITLE__" {
  state __C1__ = __D1__
  state __C2__ = __D2__
  state __C3__ = __D3__
  state saved = false

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__SUBTITLE__" color: #64748b

    rect padding: 16px, color: #f8fafc, radius: 8px {
      column gap: 12px {
        checkbox bind: __C1__, label: "__L1__"
        checkbox bind: __C2__, label: "__L2__"
        checkbox bind: __C3__, label: "__L3__"
      }
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: __CLR__, radius: 4px {
        text "Save" color: #ffffff
        on click: set saved = true
      }
    }

    if saved {
      text "Settings saved!" color: #16a34a
    }
  }
}"""

for n, cfg in [
    ("notif-email", {"TITLE": "Email Notifications", "DESC": "Email notification preferences", "SUBTITLE": "Choose which emails to receive", "C1": "marketing", "D1": "true", "L1": "Marketing emails", "C2": "updates", "D2": "true", "L2": "Product updates", "C3": "weekly-digest", "D3": "false", "L3": "Weekly digest", "CLR": "#2563eb"}),
    ("notif-push", {"TITLE": "Push Notifications", "DESC": "Push notification settings", "SUBTITLE": "Manage push notifications", "C1": "new-messages", "D1": "true", "L1": "New messages", "C2": "mentions", "D2": "true", "L2": "Mentions and replies", "C3": "promotions", "D3": "false", "L3": "Promotional alerts", "CLR": "#16a34a"}),
    ("privacy-data", {"TITLE": "Privacy Settings", "DESC": "Data privacy configuration", "SUBTITLE": "Control your data", "C1": "analytics", "D1": "true", "L1": "Usage analytics", "C2": "personalization", "D2": "false", "L2": "Personalized content", "C3": "third-party", "D3": "false", "L3": "Third-party sharing", "CLR": "#8b5cf6"}),
    ("display-prefs", {"TITLE": "Display Preferences", "DESC": "Display configuration settings", "SUBTITLE": "Customize your view", "C1": "compact-mode", "D1": "false", "L1": "Compact mode", "C2": "show-avatars", "D2": "true", "L2": "Show user avatars", "C3": "animations-on", "D3": "true", "L3": "Enable animations", "CLR": "#f59e0b"}),
    ("security-2fa", {"TITLE": "Security Settings", "DESC": "Security and authentication", "SUBTITLE": "Protect your account", "C1": "two-factor", "D1": "false", "L1": "Two-factor authentication", "C2": "login-alerts", "D2": "true", "L2": "Login alerts", "C3": "session-timeout", "D3": "true", "L3": "Auto-logout after inactivity", "CLR": "#ef4444"}),
    ("editor-cfg", {"TITLE": "Editor Settings", "DESC": "Code editor preferences", "SUBTITLE": "Configure your editor", "C1": "line-numbers", "D1": "true", "L1": "Show line numbers", "C2": "word-wrap", "D2": "false", "L2": "Word wrap", "C3": "auto-save", "D3": "true", "L3": "Auto-save changes", "CLR": "#06b6d4"}),
    ("a11y-cfg", {"TITLE": "Accessibility", "DESC": "Accessibility settings", "SUBTITLE": "Make the app work for you", "C1": "high-contrast", "D1": "false", "L1": "High contrast mode", "C2": "reduce-motion", "D2": "false", "L2": "Reduce motion", "C3": "screen-reader", "D3": "false", "L3": "Screen reader hints", "CLR": "#7c3aed"}),
    ("backup-cfg", {"TITLE": "Backup Settings", "DESC": "Backup configuration", "SUBTITLE": "Manage automatic backups", "C1": "auto-backup", "D1": "true", "L1": "Automatic backups", "C2": "include-media", "D2": "false", "L2": "Include media files", "C3": "encrypt-backup", "D3": "true", "L3": "Encrypt backup data", "CLR": "#0891b2"}),
    ("feed-cfg", {"TITLE": "Feed Settings", "DESC": "News feed preferences", "SUBTITLE": "Customize your feed", "C1": "show-images", "D1": "true", "L1": "Show images in feed", "C2": "auto-play", "D2": "false", "L2": "Auto-play videos", "C3": "show-previews", "D3": "true", "L3": "Show link previews", "CLR": "#ec4899"}),
    ("cookie-cfg", {"TITLE": "Cookie Preferences", "DESC": "Cookie consent settings", "SUBTITLE": "Manage cookie preferences", "C1": "necessary", "D1": "true", "L1": "Necessary cookies", "C2": "analytics-cookies", "D2": "false", "L2": "Analytics cookies", "C3": "marketing-cookies", "D3": "false", "L3": "Marketing cookies", "CLR": "#64748b"}),
]:
    ex(f"gen-check-{n}.naze", cfg["DESC"], fill(CHECK_T, cfg))

# ─── Status badges (8) ──────────────────────────────────────────────────────

BADGE_T = """-- __DESC__
app "__TITLE__" {
  state status = "__DEFAULT__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    row gap: 8px {
      rect width: 80px, height: 28px, color: __C1__, radius: 12px {
        text "__S1__" color: #ffffff, font-size: 12px
        on click: set status = "__S1__"
      }
      rect width: 80px, height: 28px, color: __C2__, radius: 12px {
        text "__S2__" color: #ffffff, font-size: 12px
        on click: set status = "__S2__"
      }
      rect width: 80px, height: 28px, color: __C3__, radius: 12px {
        text "__S3__" color: #ffffff, font-size: 12px
        on click: set status = "__S3__"
      }
    }

    text "Current: {status}" font-size: 18px, font-weight: bold
  }
}"""

for n, cfg in [
    ("task-status", {"TITLE": "Task Status", "DESC": "Task status badge selector", "DEFAULT": "todo", "S1": "todo", "C1": "#64748b", "S2": "doing", "C2": "#f59e0b", "S3": "done", "C3": "#16a34a"}),
    ("order-status", {"TITLE": "Order Status", "DESC": "Order status tracker", "DEFAULT": "pending", "S1": "pending", "C1": "#f59e0b", "S2": "shipped", "C2": "#2563eb", "S3": "delivered", "C3": "#16a34a"}),
    ("pr-status", {"TITLE": "PR Status", "DESC": "Pull request status badges", "DEFAULT": "draft", "S1": "draft", "C1": "#64748b", "S2": "review", "C2": "#8b5cf6", "S3": "merged", "C3": "#16a34a"}),
    ("health", {"TITLE": "Service Health", "DESC": "Service health indicator", "DEFAULT": "healthy", "S1": "healthy", "C1": "#16a34a", "S2": "degraded", "C2": "#f59e0b", "S3": "outage", "C3": "#ef4444"}),
    ("mood", {"TITLE": "Mood Tracker", "DESC": "Daily mood tracking badges", "DEFAULT": "neutral", "S1": "happy", "C1": "#16a34a", "S2": "neutral", "C2": "#f59e0b", "S3": "tired", "C3": "#6366f1"}),
    ("risk", {"TITLE": "Risk Level", "DESC": "Risk assessment badges", "DEFAULT": "low", "S1": "low", "C1": "#16a34a", "S2": "medium", "C2": "#f59e0b", "S3": "high", "C3": "#ef4444"}),
    ("approval", {"TITLE": "Approval Status", "DESC": "Document approval workflow", "DEFAULT": "pending", "S1": "pending", "C1": "#f59e0b", "S2": "approved", "C2": "#16a34a", "S3": "rejected", "C3": "#ef4444"}),
    ("build", {"TITLE": "Build Status", "DESC": "CI build status indicator", "DEFAULT": "running", "S1": "passed", "C1": "#16a34a", "S2": "running", "C2": "#2563eb", "S3": "failed", "C3": "#ef4444"}),
]:
    ex(f"gen-badge-{n}.naze", cfg["DESC"], fill(BADGE_T, cfg))

# ─── Countdown/progress patterns (5) ────────────────────────────────────────

ex("gen-progress-bar.naze", "Progress bar with percentage display",
   """-- Progress bar
app "Progress" {
  state progress = 65

  column padding: 20px, gap: 16px {
    heading "Upload Progress"
    text "{progress}%" font-size: 24px, color: #2563eb

    rect width: 300px, height: 20px, color: #e2e8f0, radius: 10px {
      rect width: 195px, height: 20px, color: #2563eb, radius: 10px
    }

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "+10" color: #ffffff
        on click: set progress = progress + 10
      }
      rect width: 60px, height: 32px, color: #ef4444, radius: 4px {
        text "Reset" color: #ffffff
        on click: set progress = 0
      }
    }
  }
}""")

ex("gen-step-indicator.naze", "Step indicator for multi-step process",
   """-- Step indicator
app "Steps" {
  state step = 1

  column padding: 20px, gap: 16px {
    heading "Setup Wizard"

    row gap: 16px {
      rect width: 32px, height: 32px, color: #2563eb, radius: 16px {
        text "1" color: #ffffff
      }
      rect width: 32px, height: 32px, color: #e2e8f0, radius: 16px {
        text "2" color: #64748b
      }
      rect width: 32px, height: 32px, color: #e2e8f0, radius: 16px {
        text "3" color: #64748b
      }
    }

    text "Step {step} of 3" color: #64748b

    match step {
      1: text "Enter your details" font-size: 18px
      2: text "Choose your plan" font-size: 18px
      3: text "Confirm and finish" font-size: 18px
      _: text "Complete" color: #16a34a
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "Back" color: #ffffff
        on click: set step = step - 1
      }
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "Next" color: #ffffff
        on click: set step = step + 1
      }
    }
  }
}""")

ex("gen-loading-skeleton.naze", "Loading skeleton placeholder UI",
   """-- Loading skeleton
app "Loading" {
  state loaded = false

  column padding: 20px, gap: 16px {
    heading "Content"

    if loaded {
      text "Content has loaded!" font-size: 18px
      text "Here is your data." color: #64748b
    }

    if loaded == false {
      rect width: 300px, height: 20px, color: #e2e8f0, radius: 4px
      rect width: 250px, height: 16px, color: #e2e8f0, radius: 4px
      rect width: 200px, height: 16px, color: #e2e8f0, radius: 4px
    }

    rect width: 100px, height: 36px, color: #2563eb, radius: 4px {
      text "Load" color: #ffffff
      on click: set loaded = true
    }
  }
}""")

ex("gen-rating-display.naze", "Star rating display with click",
   """-- Star rating
app "Rating" {
  state rating = 3

  column padding: 20px, gap: 16px {
    heading "Rate this product"
    text "Rating: {rating} / 5" font-size: 18px

    row gap: 4px {
      rect width: 40px, height: 40px, color: #fbbf24, radius: 4px {
        text "1" color: #ffffff
        on click: set rating = 1
      }
      rect width: 40px, height: 40px, color: #fbbf24, radius: 4px {
        text "2" color: #ffffff
        on click: set rating = 2
      }
      rect width: 40px, height: 40px, color: #fbbf24, radius: 4px {
        text "3" color: #ffffff
        on click: set rating = 3
      }
      rect width: 40px, height: 40px, color: #d1d5db, radius: 4px {
        text "4" color: #ffffff
        on click: set rating = 4
      }
      rect width: 40px, height: 40px, color: #d1d5db, radius: 4px {
        text "5" color: #ffffff
        on click: set rating = 5
      }
    }

    match rating {
      1: text "Poor" color: #ef4444
      2: text "Fair" color: #f97316
      3: text "Good" color: #eab308
      4: text "Great" color: #22c55e
      5: text "Excellent" color: #16a34a
      _: text "Rate us!" color: #64748b
    }
  }
}""")

ex("gen-breadcrumb-nav.naze", "Breadcrumb navigation with state",
   """-- Breadcrumb nav
app "Breadcrumbs" {
  state section = "home"

  column padding: 20px, gap: 16px {
    row gap: 4px {
      text "Home" color: #2563eb
      text ">" color: #94a3b8
      text "{section}" color: #334155, font-weight: bold
    }

    separator

    row gap: 8px {
      rect width: 80px, height: 32px, color: #eff6ff, radius: 4px {
        text "Products" color: #2563eb
        on click: set section = "products"
      }
      rect width: 80px, height: 32px, color: #eff6ff, radius: 4px {
        text "About" color: #2563eb
        on click: set section = "about"
      }
      rect width: 80px, height: 32px, color: #eff6ff, radius: 4px {
        text "Contact" color: #2563eb
        on click: set section = "contact"
      }
    }

    match section {
      "home": text "Welcome to our store" font-size: 18px
      "products": text "Browse our catalog" font-size: 18px
      "about": text "Learn about us" font-size: 18px
      "contact": text "Get in touch" font-size: 18px
      _: text "Page not found" color: #ef4444
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# Validation & main
# ═══════════════════════════════════════════════════════════════════════════════


def build_nazec():
    print("Building nazec...", end=" ", flush=True)
    result = subprocess.run(
        ["cargo", "build", "-p", "nazec", "--quiet"],
        capture_output=True, text=True, cwd=PROJECT_ROOT,
    )
    if result.returncode != 0:
        print("FAILED")
        print(result.stderr)
        sys.exit(1)
    print("OK")


def validate(filepath):
    result = subprocess.run(
        [str(NAZEC), "parse", str(filepath)],
        capture_output=True, text=True,
    )
    return result.returncode == 0, result.stderr.strip()



# ═══════════════════════════════════════════════════════════════════════════════
# Batch A: 100 training examples across 10 new categories
# This file is spliced into generate_examples.py — do NOT add imports or main()
# ═══════════════════════════════════════════════════════════════════════════════


# ─── 1. Profile pages (gen-profile-*) — 12 examples ──────────────────────────

PROFILE_T = """-- __DESC__
app "__TITLE__" {
  state username = "__USER__"
  state bio = "__BIO__"
  state followers = __FOLLOWERS__
  state following = __FOLLOWING__
  computed ratio = followers / following

  column padding: 24px, gap: 16px {
    heading "__TITLE__"

    row gap: 16px {
      rect width: 80px, height: 80px, color: __AVATAR_CLR__, radius: 40px {
        text "__INITIAL__" color: #ffffff, font-size: 32px
      }
      column gap: 4px {
        text "{username}" font-size: 22px, font-weight: bold
        text "{bio}" color: #64748b
      }
    }

    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Followers" color: #64748b, font-size: 12px
        text "{followers}" font-size: 20px, color: __CLR__
      }
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Following" color: #64748b, font-size: 12px
        text "{following}" font-size: 20px, color: __CLR__
      }
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Ratio" color: #64748b, font-size: 12px
        text "{ratio}" font-size: 20px, color: __CLR__
      }
    }

    separator

    rect padding: 12px, color: __LINK_BG__, radius: 8px {
      link "__LINK_TEXT__" href: "__LINK_URL__"
    }
  }
}"""

for n, cfg in [
    ("dev", {"TITLE": "Dev Profile", "DESC": "Developer profile with GitHub link",
             "USER": "alice_dev", "BIO": "Full-stack developer", "FOLLOWERS": "1240",
             "FOLLOWING": "310", "INITIAL": "A", "AVATAR_CLR": "#3b82f6",
             "CLR": "#2563eb", "LINK_BG": "#eff6ff",
             "LINK_TEXT": "GitHub", "LINK_URL": "https://github.com"}),
    ("designer", {"TITLE": "Designer Profile", "DESC": "UI designer portfolio profile",
                  "USER": "maria_ux", "BIO": "UI/UX designer and illustrator", "FOLLOWERS": "3400",
                  "FOLLOWING": "180", "INITIAL": "M", "AVATAR_CLR": "#ec4899",
                  "CLR": "#db2777", "LINK_BG": "#fdf2f8",
                  "LINK_TEXT": "Dribbble", "LINK_URL": "https://dribbble.com"}),
    ("writer", {"TITLE": "Writer Profile", "DESC": "Content writer profile page",
                "USER": "john_writes", "BIO": "Technical writer and blogger", "FOLLOWERS": "890",
                "FOLLOWING": "420", "INITIAL": "J", "AVATAR_CLR": "#8b5cf6",
                "CLR": "#7c3aed", "LINK_BG": "#f5f3ff",
                "LINK_TEXT": "Blog", "LINK_URL": "https://medium.com"}),
    ("artist", {"TITLE": "Artist Profile", "DESC": "Digital artist showcase profile",
                "USER": "luna_art", "BIO": "Digital artist and animator", "FOLLOWERS": "5600",
                "FOLLOWING": "95", "INITIAL": "L", "AVATAR_CLR": "#f59e0b",
                "CLR": "#d97706", "LINK_BG": "#fef3c7",
                "LINK_TEXT": "Portfolio", "LINK_URL": "https://artstation.com"}),
    ("chef", {"TITLE": "Chef Profile", "DESC": "Chef profile with recipe link",
              "USER": "chef_marco", "BIO": "Italian cuisine enthusiast", "FOLLOWERS": "2100",
              "FOLLOWING": "150", "INITIAL": "C", "AVATAR_CLR": "#ef4444",
              "CLR": "#dc2626", "LINK_BG": "#fef2f2",
              "LINK_TEXT": "Recipes", "LINK_URL": "https://allrecipes.com"}),
    ("coach", {"TITLE": "Coach Profile", "DESC": "Fitness coach profile page",
               "USER": "fit_sam", "BIO": "Certified personal trainer", "FOLLOWERS": "4300",
               "FOLLOWING": "220", "INITIAL": "S", "AVATAR_CLR": "#16a34a",
               "CLR": "#15803d", "LINK_BG": "#f0fdf4",
               "LINK_TEXT": "Programs", "LINK_URL": "https://fitcoach.com"}),
    ("musician", {"TITLE": "Musician Profile", "DESC": "Musician profile with Spotify link",
                  "USER": "beats_kai", "BIO": "Producer and singer-songwriter", "FOLLOWERS": "7800",
                  "FOLLOWING": "60", "INITIAL": "K", "AVATAR_CLR": "#06b6d4",
                  "CLR": "#0891b2", "LINK_BG": "#ecfeff",
                  "LINK_TEXT": "Spotify", "LINK_URL": "https://spotify.com"}),
    ("teacher", {"TITLE": "Teacher Profile", "DESC": "Online teacher profile page",
                 "USER": "prof_nina", "BIO": "Math teacher and tutor", "FOLLOWERS": "1560",
                 "FOLLOWING": "340", "INITIAL": "N", "AVATAR_CLR": "#6366f1",
                 "CLR": "#4f46e5", "LINK_BG": "#eef2ff",
                 "LINK_TEXT": "Courses", "LINK_URL": "https://udemy.com"}),
    ("photographer", {"TITLE": "Photographer Profile", "DESC": "Photographer with gallery link",
                      "USER": "lens_emma", "BIO": "Landscape and portrait photographer", "FOLLOWERS": "9200",
                      "FOLLOWING": "130", "INITIAL": "E", "AVATAR_CLR": "#14b8a6",
                      "CLR": "#0d9488", "LINK_BG": "#f0fdfa",
                      "LINK_TEXT": "Gallery", "LINK_URL": "https://500px.com"}),
    ("gamer", {"TITLE": "Gamer Profile", "DESC": "Esports player profile",
               "USER": "pro_rex", "BIO": "Competitive FPS player", "FOLLOWERS": "12000",
               "FOLLOWING": "45", "INITIAL": "R", "AVATAR_CLR": "#a855f7",
               "CLR": "#9333ea", "LINK_BG": "#faf5ff",
               "LINK_TEXT": "Twitch", "LINK_URL": "https://twitch.tv"}),
    ("scientist", {"TITLE": "Scientist Profile", "DESC": "Research scientist profile",
                   "USER": "dr_chen", "BIO": "AI researcher and professor", "FOLLOWERS": "3100",
                   "FOLLOWING": "270", "INITIAL": "D", "AVATAR_CLR": "#0ea5e9",
                   "CLR": "#0284c7", "LINK_BG": "#f0f9ff",
                   "LINK_TEXT": "Papers", "LINK_URL": "https://scholar.google.com"}),
    ("streamer", {"TITLE": "Streamer Profile", "DESC": "Live streamer profile page",
                  "USER": "live_zoe", "BIO": "Variety streamer and content creator", "FOLLOWERS": "18500",
                  "FOLLOWING": "75", "INITIAL": "Z", "AVATAR_CLR": "#d946ef",
                  "CLR": "#c026d3", "LINK_BG": "#fdf4ff",
                  "LINK_TEXT": "YouTube", "LINK_URL": "https://youtube.com"}),
]:
    ex(f"gen-profile-{n}.naze", cfg["DESC"], fill(PROFILE_T, cfg))


# ─── 2. Invoice / receipt layouts (gen-invoice-*) — 10 examples ──────────────

INVOICE_T = """-- __DESC__
app "__TITLE__" {
  state item1-qty = __Q1__
  state item2-qty = __Q2__
  state item1-price = __P1__
  state item2-price = __P2__
  state tax-rate = __TAX__
  computed subtotal = item1-qty * item1-price + item2-qty * item2-price
  computed tax = subtotal * tax-rate / 100
  computed total = subtotal + tax

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    rect padding: 16px, color: #f8fafc, radius: 8px {
      column gap: 8px {
        text "__VENDOR__" font-size: 18px, font-weight: bold
        text "__VADDR__" color: #64748b, font-size: 13px
        separator
        text "Invoice #__INVNUM__" color: #94a3b8, font-size: 12px
      }
    }

    column gap: 4px {
      row gap: 8px {
        text "__I1__" font-weight: bold
        spacer
        text "{item1-qty} x ${item1-price}" color: #64748b
      }
      row gap: 8px {
        text "__I2__" font-weight: bold
        spacer
        text "{item2-qty} x ${item2-price}" color: #64748b
      }
    }

    separator

    column gap: 4px {
      row gap: 8px {
        text "Subtotal" color: #64748b
        spacer
        text "${subtotal}"
      }
      row gap: 8px {
        text "Tax ({tax-rate}%)" color: #64748b
        spacer
        text "${tax}"
      }
      row gap: 8px {
        text "Total" font-weight: bold, font-size: 18px
        spacer
        text "${total}" font-weight: bold, font-size: 18px, color: __CLR__
      }
    }
  }
}"""

for n, cfg in [
    ("cafe", {"TITLE": "Cafe Receipt", "DESC": "Coffee shop receipt with tax",
              "VENDOR": "Bean & Brew Cafe", "VADDR": "123 Coffee Lane",
              "INVNUM": "CB-4021", "I1": "Latte", "I2": "Muffin",
              "Q1": "2", "Q2": "1", "P1": "5", "P2": "4", "TAX": "8", "CLR": "#92400e"}),
    ("electronics", {"TITLE": "Tech Invoice", "DESC": "Electronics store invoice",
                     "VENDOR": "TechMart", "VADDR": "456 Silicon Blvd",
                     "INVNUM": "TM-8833", "I1": "Keyboard", "I2": "Mouse",
                     "Q1": "1", "Q2": "1", "P1": "120", "P2": "60", "TAX": "10", "CLR": "#1e40af"}),
    ("restaurant", {"TITLE": "Dinner Bill", "DESC": "Restaurant dinner bill with gratuity",
                    "VENDOR": "La Tavola", "VADDR": "789 Main St",
                    "INVNUM": "LT-1157", "I1": "Pasta Primavera", "I2": "Tiramisu",
                    "Q1": "2", "Q2": "2", "P1": "18", "P2": "9", "TAX": "9", "CLR": "#b91c1c"}),
    ("freelance", {"TITLE": "Freelance Invoice", "DESC": "Freelance development invoice",
                   "VENDOR": "CodeWorks LLC", "VADDR": "Remote",
                   "INVNUM": "CW-2200", "I1": "Dev Hours", "I2": "Design Hours",
                   "Q1": "40", "Q2": "10", "P1": "150", "P2": "120", "TAX": "0", "CLR": "#4f46e5"}),
    ("grocery", {"TITLE": "Grocery Receipt", "DESC": "Grocery store receipt",
                 "VENDOR": "Fresh Market", "VADDR": "22 Oak Avenue",
                 "INVNUM": "FM-7750", "I1": "Organic Eggs", "I2": "Bread Loaf",
                 "Q1": "2", "Q2": "3", "P1": "6", "P2": "4", "TAX": "5", "CLR": "#15803d"}),
    ("salon", {"TITLE": "Salon Receipt", "DESC": "Hair salon service receipt",
               "VENDOR": "Style Studio", "VADDR": "55 Beauty Pkwy",
               "INVNUM": "SS-3390", "I1": "Haircut", "I2": "Coloring",
               "Q1": "1", "Q2": "1", "P1": "45", "P2": "85", "TAX": "7", "CLR": "#be185d"}),
    ("mechanic", {"TITLE": "Auto Repair", "DESC": "Auto repair shop invoice",
                  "VENDOR": "QuickFix Auto", "VADDR": "100 Garage Rd",
                  "INVNUM": "QF-5511", "I1": "Oil Change", "I2": "Brake Pads",
                  "Q1": "1", "Q2": "2", "P1": "50", "P2": "80", "TAX": "6", "CLR": "#475569"}),
    ("hotel", {"TITLE": "Hotel Bill", "DESC": "Hotel stay invoice with room charges",
               "VENDOR": "Grand Plaza Hotel", "VADDR": "1 Harbor View",
               "INVNUM": "GP-9001", "I1": "Nights", "I2": "Room Service",
               "Q1": "3", "Q2": "2", "P1": "200", "P2": "35", "TAX": "12", "CLR": "#7e22ce"}),
    ("vet", {"TITLE": "Vet Invoice", "DESC": "Veterinary clinic invoice",
             "VENDOR": "PetCare Clinic", "VADDR": "8 Paw St",
             "INVNUM": "PC-4400", "I1": "Checkup", "I2": "Vaccination",
             "Q1": "1", "Q2": "2", "P1": "75", "P2": "40", "TAX": "0", "CLR": "#0d9488"}),
    ("print", {"TITLE": "Print Shop Invoice", "DESC": "Print shop order receipt",
               "VENDOR": "InkPress Studio", "VADDR": "300 Print Ave",
               "INVNUM": "IP-6620", "I1": "Posters", "I2": "Business Cards",
               "Q1": "10", "Q2": "200", "P1": "8", "P2": "1", "TAX": "7", "CLR": "#ea580c"}),
]:
    ex(f"gen-invoice-{n}.naze", cfg["DESC"], fill(INVOICE_T, cfg))


# ─── 3. Weather displays (gen-weather-*) — 10 examples ──────────────────────

WEATHER_T = """-- __DESC__
app "__TITLE__" {
  state temp = __TEMP__
  state feels-like = __FEELS__
  state humidity = __HUM__
  state wind = __WIND__
  state condition = "__COND__"

  column padding: 24px, gap: 16px, color: __BG__ {
    heading "__CITY__ Weather" color: __CLR__

    row gap: 16px {
      rect width: 80px, height: 80px, color: __ICON_BG__, radius: 12px {
        text "__ICON__" font-size: 36px
      }
      column gap: 4px {
        text "{temp}__UNIT__" font-size: 48px, font-weight: bold, color: __CLR__
        text "{condition}" color: #64748b, font-size: 16px
      }
    }

    grid columns: 3, gap: 12px {
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Feels Like" color: #64748b, font-size: 11px
        text "{feels-like}__UNIT__" font-size: 18px
      }
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Humidity" color: #64748b, font-size: 11px
        text "{humidity}%" font-size: 18px
      }
      rect padding: 12px, color: #f1f5f9, radius: 8px {
        text "Wind" color: #64748b, font-size: 11px
        text "{wind} __WSPD__" font-size: 18px
      }
    }

    separator

    text "Forecast: __FORECAST__" color: #94a3b8, font-size: 13px
  }
}"""

for n, cfg in [
    ("sunny", {"TITLE": "Sunny Day", "DESC": "Clear sunny weather display",
               "CITY": "Phoenix", "TEMP": "38", "FEELS": "40", "HUM": "15", "WIND": "8",
               "COND": "Clear Sky", "ICON": "SUN", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#fef3c7", "BG": "#fffbeb", "CLR": "#d97706",
               "FORECAST": "Sunny all week, highs near 40C"}),
    ("rainy", {"TITLE": "Rainy Day", "DESC": "Rainy weather with high humidity",
               "CITY": "Seattle", "TEMP": "12", "FEELS": "10", "HUM": "92", "WIND": "18",
               "COND": "Heavy Rain", "ICON": "RN", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#dbeafe", "BG": "#f0f9ff", "CLR": "#1d4ed8",
               "FORECAST": "Rain continues through Thursday"}),
    ("snowy", {"TITLE": "Snow Report", "DESC": "Snowy winter weather display",
               "CITY": "Aspen", "TEMP": "-5", "FEELS": "-12", "HUM": "80", "WIND": "25",
               "COND": "Heavy Snow", "ICON": "SNW", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#e0e7ff", "BG": "#eef2ff", "CLR": "#4338ca",
               "FORECAST": "15cm expected overnight, powder alert"}),
    ("cloudy", {"TITLE": "Overcast", "DESC": "Cloudy overcast weather",
                "CITY": "London", "TEMP": "14", "FEELS": "13", "HUM": "75", "WIND": "12",
                "COND": "Overcast", "ICON": "CLD", "UNIT": "C", "WSPD": "mph",
                "ICON_BG": "#e2e8f0", "BG": "#f8fafc", "CLR": "#475569",
                "FORECAST": "Clouds clearing by weekend"}),
    ("windy", {"TITLE": "Wind Advisory", "DESC": "Windy weather with gusts",
               "CITY": "Chicago", "TEMP": "8", "FEELS": "2", "HUM": "45", "WIND": "55",
               "COND": "Strong Winds", "ICON": "WND", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#cffafe", "BG": "#ecfeff", "CLR": "#0e7490",
               "FORECAST": "Gusts up to 70 km/h expected"}),
    ("tropical", {"TITLE": "Tropical Heat", "DESC": "Hot tropical weather display",
                  "CITY": "Miami", "TEMP": "92", "FEELS": "98", "HUM": "88", "WIND": "5",
                  "COND": "Hot and Humid", "ICON": "HOT", "UNIT": "F", "WSPD": "mph",
                  "ICON_BG": "#fecaca", "BG": "#fff1f2", "CLR": "#dc2626",
                  "FORECAST": "Heat advisory through Friday"}),
    ("foggy", {"TITLE": "Fog Alert", "DESC": "Foggy morning weather display",
               "CITY": "San Francisco", "TEMP": "15", "FEELS": "14", "HUM": "95", "WIND": "6",
               "COND": "Dense Fog", "ICON": "FOG", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#f1f5f9", "BG": "#f8fafc", "CLR": "#64748b",
               "FORECAST": "Fog burns off by noon, sunny afternoon"}),
    ("storm", {"TITLE": "Storm Warning", "DESC": "Thunderstorm weather alert",
               "CITY": "Dallas", "TEMP": "28", "FEELS": "32", "HUM": "70", "WIND": "40",
               "COND": "Thunderstorms", "ICON": "STM", "UNIT": "C", "WSPD": "km/h",
               "ICON_BG": "#fef9c3", "BG": "#fefce8", "CLR": "#a16207",
               "FORECAST": "Severe storms possible this evening"}),
    ("mild", {"TITLE": "Pleasant Day", "DESC": "Mild pleasant weather display",
              "CITY": "Barcelona", "TEMP": "22", "FEELS": "22", "HUM": "50", "WIND": "10",
              "COND": "Partly Cloudy", "ICON": "MLD", "UNIT": "C", "WSPD": "km/h",
              "ICON_BG": "#d1fae5", "BG": "#ecfdf5", "CLR": "#059669",
              "FORECAST": "Beautiful weather all week"}),
    ("cold", {"TITLE": "Cold Snap", "DESC": "Extreme cold weather warning",
              "CITY": "Winnipeg", "TEMP": "-28", "FEELS": "-38", "HUM": "60", "WIND": "30",
              "COND": "Extreme Cold", "ICON": "ICE", "UNIT": "C", "WSPD": "km/h",
              "ICON_BG": "#bae6fd", "BG": "#f0f9ff", "CLR": "#0369a1",
              "FORECAST": "Windchill warning in effect until Thursday"}),
]:
    ex(f"gen-weather-{n}.naze", cfg["DESC"], fill(WEATHER_T, cfg))


# ─── 4. Restaurant / app menus (gen-menu-*) — 10 examples ───────────────────

MENU_T = """-- __DESC__
app "__TITLE__" {
  state category = "__CAT1__"
  state items = [__ITEMS__]
  computed total-items = items | count

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    row gap: 8px {
      rect padding: 8px, color: __CLR__, radius: 4px {
        text "__CAT1__" color: #ffffff, font-size: 13px
        on click: set category = "__CAT1__"
      }
      rect padding: 8px, color: #e2e8f0, radius: 4px {
        text "__CAT2__" font-size: 13px
        on click: set category = "__CAT2__"
      }
      rect padding: 8px, color: #e2e8f0, radius: 4px {
        text "__CAT3__" font-size: 13px
        on click: set category = "__CAT3__"
      }
    }

    text "{total-items} items on the menu" color: #94a3b8, font-size: 12px

    each item in items {
      rect padding: 12px, color: __ITEM_BG__, radius: 8px {
        row gap: 8px {
          column gap: 2px {
            text "{item.name}" font-weight: bold
            text "{item.desc}" color: #64748b, font-size: 13px
          }
          spacer
          text "{item.price}" font-weight: bold, color: __CLR__
        }
      }
    }
  }
}"""

for n, cfg in [
    ("italian", {"TITLE": "Trattoria Menu", "DESC": "Italian restaurant menu with categories",
                 "CAT1": "Pasta", "CAT2": "Pizza", "CAT3": "Desserts",
                 "ITEMS": '{name: "Carbonara", desc: "Egg, pecorino, guanciale", price: "$16"}, {name: "Margherita", desc: "Tomato, mozzarella, basil", price: "$14"}, {name: "Tiramisu", desc: "Coffee-soaked ladyfingers", price: "$9"}',
                 "CLR": "#b91c1c", "ITEM_BG": "#fef2f2"}),
    ("sushi", {"TITLE": "Sushi Bar", "DESC": "Japanese sushi restaurant menu",
               "CAT1": "Nigiri", "CAT2": "Rolls", "CAT3": "Drinks",
               "ITEMS": '{name: "Salmon Nigiri", desc: "Fresh Atlantic salmon", price: "$6"}, {name: "Dragon Roll", desc: "Eel, avocado, cucumber", price: "$14"}, {name: "Matcha Latte", desc: "Ceremonial grade matcha", price: "$5"}',
               "CLR": "#0f766e", "ITEM_BG": "#f0fdfa"}),
    ("burger", {"TITLE": "Burger Joint", "DESC": "Burger restaurant menu",
                "CAT1": "Burgers", "CAT2": "Sides", "CAT3": "Shakes",
                "ITEMS": '{name: "Classic Smash", desc: "Double patty, cheese, pickles", price: "$12"}, {name: "Loaded Fries", desc: "Cheese, bacon, jalapenos", price: "$8"}, {name: "Vanilla Shake", desc: "Hand-spun ice cream", price: "$6"}',
                "CLR": "#c2410c", "ITEM_BG": "#fff7ed"}),
    ("cafe", {"TITLE": "Cafe Menu", "DESC": "Coffee shop menu with pastries",
              "CAT1": "Coffee", "CAT2": "Tea", "CAT3": "Pastries",
              "ITEMS": '{name: "Espresso", desc: "Double shot, house blend", price: "$4"}, {name: "Chai Latte", desc: "Spiced masala chai", price: "$5"}, {name: "Croissant", desc: "Butter, flaky, fresh baked", price: "$4"}',
              "CLR": "#78350f", "ITEM_BG": "#fef3c7"}),
    ("thai", {"TITLE": "Thai Kitchen", "DESC": "Thai restaurant menu with spice levels",
              "CAT1": "Curries", "CAT2": "Noodles", "CAT3": "Appetizers",
              "ITEMS": '{name: "Green Curry", desc: "Coconut milk, bamboo shoots", price: "$15"}, {name: "Pad Thai", desc: "Rice noodles, shrimp, peanuts", price: "$14"}, {name: "Spring Rolls", desc: "Vegetables, sweet chili sauce", price: "$7"}',
              "CLR": "#15803d", "ITEM_BG": "#f0fdf4"}),
    ("pizza", {"TITLE": "Slice House", "DESC": "Pizza parlor menu",
               "CAT1": "Classics", "CAT2": "Specialty", "CAT3": "Sides",
               "ITEMS": '{name: "Pepperoni", desc: "Mozzarella, pepperoni, oregano", price: "$18"}, {name: "BBQ Chicken", desc: "BBQ sauce, red onion, cilantro", price: "$20"}, {name: "Garlic Knots", desc: "Butter, parmesan, herbs", price: "$6"}',
               "CLR": "#dc2626", "ITEM_BG": "#fef2f2"}),
    ("taco", {"TITLE": "Taco Stand", "DESC": "Mexican taco menu",
              "CAT1": "Tacos", "CAT2": "Burritos", "CAT3": "Drinks",
              "ITEMS": '{name: "Al Pastor", desc: "Pineapple, onion, cilantro", price: "$4"}, {name: "Carne Asada", desc: "Grilled steak, guacamole", price: "$5"}, {name: "Horchata", desc: "Rice milk, cinnamon, vanilla", price: "$3"}',
              "CLR": "#ea580c", "ITEM_BG": "#fff7ed"}),
    ("vegan", {"TITLE": "Green Plate", "DESC": "Vegan restaurant menu",
               "CAT1": "Mains", "CAT2": "Bowls", "CAT3": "Smoothies",
               "ITEMS": '{name: "Tempeh Steak", desc: "Marinated, grilled, chimichurri", price: "$16"}, {name: "Buddha Bowl", desc: "Quinoa, avocado, tahini", price: "$14"}, {name: "Berry Blast", desc: "Mixed berries, banana, oat milk", price: "$8"}',
               "CLR": "#16a34a", "ITEM_BG": "#f0fdf4"}),
    ("seafood", {"TITLE": "The Catch", "DESC": "Seafood restaurant menu",
                 "CAT1": "Fish", "CAT2": "Shellfish", "CAT3": "Starters",
                 "ITEMS": '{name: "Grilled Salmon", desc: "Lemon herb butter, rice", price: "$24"}, {name: "Lobster Tail", desc: "Drawn butter, asparagus", price: "$38"}, {name: "Clam Chowder", desc: "New England style, sourdough", price: "$10"}',
                 "CLR": "#0369a1", "ITEM_BG": "#f0f9ff"}),
    ("bakery", {"TITLE": "Sweet Crust", "DESC": "Bakery and patisserie menu",
                "CAT1": "Bread", "CAT2": "Cakes", "CAT3": "Cookies",
                "ITEMS": '{name: "Sourdough Loaf", desc: "24-hour ferment, crusty", price: "$8"}, {name: "Chocolate Cake", desc: "Triple layer, ganache", price: "$7"}, {name: "Macarons", desc: "Assorted box of 6", price: "$12"}',
                "CLR": "#9333ea", "ITEM_BG": "#faf5ff"}),
]:
    ex(f"gen-menu-{n}.naze", cfg["DESC"], fill(MENU_T, cfg))


# ─── 5. Event timelines (gen-timeline-*) — 10 examples ──────────────────────

TIMELINE_T = """-- __DESC__
app "__TITLE__" {
  state events = [__EVENTS__]
  computed total = events | count

  column padding: 24px, gap: 8px {
    heading "__TITLE__" color: __CLR__
    text "{total} events" color: #94a3b8, font-size: 12px

    each evt in events {
      row gap: 12px {
        column gap: 0px {
          rect width: 12px, height: 12px, color: __DOT_CLR__, radius: 6px
          rect width: 2px, height: 40px, color: #e2e8f0
        }
        column gap: 2px {
          text "{evt.time}" color: #94a3b8, font-size: 11px
          text "{evt.title}" font-weight: bold
          text "{evt.detail}" color: #64748b, font-size: 13px
        }
      }
    }
  }
}"""

for n, cfg in [
    ("project", {"TITLE": "Project Timeline", "DESC": "Software project milestone timeline",
                 "EVENTS": '{time: "Jan 10", title: "Kickoff", detail: "Requirements gathered"}, {time: "Feb 15", title: "Alpha", detail: "Core features complete"}, {time: "Mar 20", title: "Beta", detail: "User testing begins"}, {time: "Apr 01", title: "Launch", detail: "Public release"}',
                 "CLR": "#2563eb", "DOT_CLR": "#3b82f6"}),
    ("workday", {"TITLE": "Daily Log", "DESC": "Daily work activity timeline",
                 "EVENTS": '{time: "9:00 AM", title: "Standup", detail: "Team sync meeting"}, {time: "10:30 AM", title: "Code Review", detail: "Reviewed 3 pull requests"}, {time: "2:00 PM", title: "Deploy", detail: "v2.1 pushed to staging"}, {time: "4:30 PM", title: "Retro", detail: "Sprint retrospective"}',
                 "CLR": "#4f46e5", "DOT_CLR": "#6366f1"}),
    ("hiring", {"TITLE": "Hiring Pipeline", "DESC": "Recruitment process timeline",
                "EVENTS": '{time: "Week 1", title: "Posted", detail: "Job listing published"}, {time: "Week 3", title: "Screening", detail: "50 applications reviewed"}, {time: "Week 5", title: "Interviews", detail: "8 candidates interviewed"}, {time: "Week 6", title: "Offer", detail: "Offer extended to top candidate"}',
                "CLR": "#059669", "DOT_CLR": "#10b981"}),
    ("order", {"TITLE": "Order Tracking", "DESC": "Package delivery tracking timeline",
               "EVENTS": '{time: "Mon 8AM", title: "Ordered", detail: "Payment confirmed"}, {time: "Tue 2PM", title: "Packed", detail: "Warehouse processed"}, {time: "Thu 10AM", title: "Shipped", detail: "In transit via carrier"}, {time: "Fri 3PM", title: "Delivered", detail: "Left at front door"}',
               "CLR": "#ea580c", "DOT_CLR": "#f97316"}),
    ("education", {"TITLE": "Learning Path", "DESC": "Course progression timeline",
                   "EVENTS": '{time: "Module 1", title: "Basics", detail: "Syntax and variables"}, {time: "Module 2", title: "Functions", detail: "Closures and scoping"}, {time: "Module 3", title: "Data", detail: "Structures and algorithms"}, {time: "Module 4", title: "Project", detail: "Capstone assignment"}',
                   "CLR": "#7c3aed", "DOT_CLR": "#8b5cf6"}),
    ("startup", {"TITLE": "Startup Journey", "DESC": "Startup company milestone timeline",
                 "EVENTS": '{time: "2024 Q1", title: "Founded", detail: "Team of 3 in garage"}, {time: "2024 Q3", title: "Seed Round", detail: "$500K raised"}, {time: "2025 Q1", title: "Product", detail: "MVP launched"}, {time: "2025 Q4", title: "Series A", detail: "$5M raised"}',
                 "CLR": "#0891b2", "DOT_CLR": "#06b6d4"}),
    ("wedding", {"TITLE": "Wedding Plan", "DESC": "Wedding planning timeline",
                 "EVENTS": '{time: "12 months", title: "Venue", detail: "Booked reception hall"}, {time: "9 months", title: "Vendors", detail: "Caterer and DJ confirmed"}, {time: "6 months", title: "Invites", detail: "Save the dates sent"}, {time: "1 month", title: "Final", detail: "Rehearsal and fitting"}',
                 "CLR": "#db2777", "DOT_CLR": "#ec4899"}),
    ("fitness", {"TITLE": "Fitness Journey", "DESC": "Workout progress timeline",
                 "EVENTS": '{time: "Week 1", title: "Start", detail: "Baseline assessment"}, {time: "Week 4", title: "Progress", detail: "5kg bench increase"}, {time: "Week 8", title: "Milestone", detail: "First 5K run"}, {time: "Week 12", title: "Goal", detail: "Target weight reached"}',
                 "CLR": "#16a34a", "DOT_CLR": "#22c55e"}),
    ("release", {"TITLE": "Release History", "DESC": "Software version release timeline",
                 "EVENTS": '{time: "v1.0", title: "Initial Release", detail: "Core features only"}, {time: "v1.5", title: "Polish", detail: "Bug fixes and perf"}, {time: "v2.0", title: "Major Update", detail: "New API and plugins"}, {time: "v2.1", title: "Patch", detail: "Security fixes"}',
                 "CLR": "#475569", "DOT_CLR": "#64748b"}),
    ("travel", {"TITLE": "Trip Itinerary", "DESC": "Travel itinerary timeline",
                "EVENTS": '{time: "Day 1", title: "Arrive Tokyo", detail: "Check into hotel, explore Shibuya"}, {time: "Day 2", title: "Temples", detail: "Visit Senso-ji and Meiji Shrine"}, {time: "Day 3", title: "Hakone", detail: "Day trip, hot springs, Mt Fuji"}, {time: "Day 4", title: "Depart", detail: "Narita airport, evening flight"}',
                "CLR": "#0284c7", "DOT_CLR": "#0ea5e9"}),
]:
    ex(f"gen-timeline-{n}.naze", cfg["DESC"], fill(TIMELINE_T, cfg))


# ─── 6. Tickets (gen-ticket-*) — 10 examples ────────────────────────────────

TICKET_T = """-- __DESC__
app "__TITLE__" {
  state ticket-id = "__TID__"
  state status = "__STATUS__"
  state priority = "__PRIORITY__"

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    rect padding: 20px, color: #ffffff, radius: 12px {
      column gap: 12px {
        row gap: 8px {
          text "__TYPE__" font-weight: bold, font-size: 18px
          spacer
          rect padding: 6px, color: __STATUS_BG__, radius: 4px {
            text "{status}" color: __STATUS_CLR__, font-size: 12px
          }
        }

        separator

        row gap: 16px {
          column gap: 4px {
            text "ID" color: #94a3b8, font-size: 11px
            text "{ticket-id}" font-size: 14px
          }
          column gap: 4px {
            text "Priority" color: #94a3b8, font-size: 11px
            text "{priority}" font-size: 14px, color: __PRI_CLR__
          }
          column gap: 4px {
            text "__FIELD_LABEL__" color: #94a3b8, font-size: 11px
            text "__FIELD_VAL__" font-size: 14px
          }
        }

        separator

        text "__DETAIL1__" color: #334155
        text "__DETAIL2__" color: #64748b, font-size: 13px
      }
    }

    match status {
      "__STATUS__": rect padding: 10px, color: __STATUS_BG__, radius: 8px {
        text "__STATUS_MSG__" color: __STATUS_CLR__, font-size: 13px
      }
      _: text "Unknown status" color: #94a3b8
    }
  }
}"""

for n, cfg in [
    ("support", {"TITLE": "Support Ticket", "DESC": "Customer support ticket view",
                 "TYPE": "Bug Report", "TID": "SUP-4421", "STATUS": "Open",
                 "PRIORITY": "High", "FIELD_LABEL": "Category", "FIELD_VAL": "Login Issue",
                 "DETAIL1": "User cannot log in with SSO credentials.",
                 "DETAIL2": "Reported by: john@example.com on Jan 15",
                 "CLR": "#dc2626", "STATUS_BG": "#fef2f2", "STATUS_CLR": "#dc2626",
                 "PRI_CLR": "#dc2626", "STATUS_MSG": "Awaiting engineer assignment"}),
    ("feature", {"TITLE": "Feature Request", "DESC": "Feature request ticket tracker",
                 "TYPE": "Feature Request", "TID": "FEAT-0089", "STATUS": "In Review",
                 "PRIORITY": "Medium", "FIELD_LABEL": "Votes", "FIELD_VAL": "47",
                 "DETAIL1": "Add dark mode support to the dashboard.",
                 "DETAIL2": "Requested by: 12 users in last 30 days",
                 "CLR": "#7c3aed", "STATUS_BG": "#f5f3ff", "STATUS_CLR": "#7c3aed",
                 "PRI_CLR": "#f59e0b", "STATUS_MSG": "Under review by product team"}),
    ("concert", {"TITLE": "Concert Ticket", "DESC": "Concert event ticket display",
                 "TYPE": "Live Concert", "TID": "EVT-7720", "STATUS": "Confirmed",
                 "PRIORITY": "VIP", "FIELD_LABEL": "Venue", "FIELD_VAL": "Madison Square",
                 "DETAIL1": "The Midnight - Live Tour 2026",
                 "DETAIL2": "Saturday, March 15 at 8:00 PM - Section A, Row 5",
                 "CLR": "#0891b2", "STATUS_BG": "#ecfeff", "STATUS_CLR": "#0891b2",
                 "PRI_CLR": "#d97706", "STATUS_MSG": "E-ticket confirmed, show at entry"}),
    ("flight", {"TITLE": "Boarding Pass", "DESC": "Flight boarding pass display",
                "TYPE": "Boarding Pass", "TID": "AA-2847", "STATUS": "Checked In",
                "PRIORITY": "Economy", "FIELD_LABEL": "Gate", "FIELD_VAL": "B22",
                "DETAIL1": "New York JFK to London Heathrow",
                "DETAIL2": "Departure: Feb 20, 2026 at 7:45 PM - Seat 14C",
                "CLR": "#1e40af", "STATUS_BG": "#eff6ff", "STATUS_CLR": "#1e40af",
                "PRI_CLR": "#64748b", "STATUS_MSG": "Boarding begins at 7:15 PM"}),
    ("maintenance", {"TITLE": "Maintenance Ticket", "DESC": "Building maintenance work order",
                     "TYPE": "Work Order", "TID": "MNT-1190", "STATUS": "Scheduled",
                     "PRIORITY": "Normal", "FIELD_LABEL": "Location", "FIELD_VAL": "Floor 3",
                     "DETAIL1": "Replace HVAC filter in server room.",
                     "DETAIL2": "Estimated time: 2 hours, technician assigned",
                     "CLR": "#475569", "STATUS_BG": "#f1f5f9", "STATUS_CLR": "#475569",
                     "PRI_CLR": "#0ea5e9", "STATUS_MSG": "Scheduled for next Tuesday"}),
    ("parking", {"TITLE": "Parking Permit", "DESC": "Parking permit ticket display",
                 "TYPE": "Parking Permit", "TID": "PKG-0055", "STATUS": "Active",
                 "PRIORITY": "Monthly", "FIELD_LABEL": "Lot", "FIELD_VAL": "Garage B",
                 "DETAIL1": "Vehicle: Silver Tesla Model 3",
                 "DETAIL2": "Valid: Jan 1 - Jan 31, 2026, Spot #127",
                 "CLR": "#059669", "STATUS_BG": "#ecfdf5", "STATUS_CLR": "#059669",
                 "PRI_CLR": "#64748b", "STATUS_MSG": "Permit valid, please display on dash"}),
    ("movie", {"TITLE": "Movie Ticket", "DESC": "Cinema movie ticket",
               "TYPE": "Cinema Ticket", "TID": "CIN-3300", "STATUS": "Valid",
               "PRIORITY": "Premium", "FIELD_LABEL": "Screen", "FIELD_VAL": "IMAX 2",
               "DETAIL1": "Interstellar - Director's Cut",
               "DETAIL2": "Friday 7:30 PM - Seats J11, J12",
               "CLR": "#a21caf", "STATUS_BG": "#fdf4ff", "STATUS_CLR": "#a21caf",
               "PRI_CLR": "#d97706", "STATUS_MSG": "Present QR code at entrance"}),
    ("helpdesk", {"TITLE": "IT Helpdesk", "DESC": "IT helpdesk service ticket",
                  "TYPE": "IT Request", "TID": "IT-5560", "STATUS": "In Progress",
                  "PRIORITY": "Urgent", "FIELD_LABEL": "Assignee", "FIELD_VAL": "Dave K.",
                  "DETAIL1": "Laptop not connecting to corporate VPN.",
                  "DETAIL2": "User: Sarah M., Dept: Marketing, ext. 2401",
                  "CLR": "#ea580c", "STATUS_BG": "#fff7ed", "STATUS_CLR": "#ea580c",
                  "PRI_CLR": "#dc2626", "STATUS_MSG": "Technician en route, ETA 15 min"}),
    ("library", {"TITLE": "Library Card", "DESC": "Library book hold ticket",
                 "TYPE": "Hold Ticket", "TID": "LIB-8821", "STATUS": "Ready",
                 "PRIORITY": "Standard", "FIELD_LABEL": "Shelf", "FIELD_VAL": "Holds-C",
                 "DETAIL1": "Dune by Frank Herbert - Hardcover",
                 "DETAIL2": "Pickup by: Feb 28, 2026 - Card #44201",
                 "CLR": "#92400e", "STATUS_BG": "#fef3c7", "STATUS_CLR": "#92400e",
                 "PRI_CLR": "#64748b", "STATUS_MSG": "Ready for pickup at front desk"}),
    ("train", {"TITLE": "Train Ticket", "DESC": "Railway train ticket display",
               "TYPE": "Rail Pass", "TID": "TRN-6619", "STATUS": "Validated",
               "PRIORITY": "First Class", "FIELD_LABEL": "Platform", "FIELD_VAL": "9A",
               "DETAIL1": "Paris Gare du Nord to Amsterdam Centraal",
               "DETAIL2": "Eurostar 9143, Coach 8, Seat 42A",
               "CLR": "#1d4ed8", "STATUS_BG": "#eff6ff", "STATUS_CLR": "#1d4ed8",
               "PRI_CLR": "#d97706", "STATUS_MSG": "Board 15 minutes before departure"}),
]:
    ex(f"gen-ticket-{n}.naze", cfg["DESC"], fill(TICKET_T, cfg))


# ─── 7. Kanban / task boards (gen-board-*) — 10 examples ────────────────────

BOARD_T = """-- __DESC__
app "__TITLE__" {
  state todo = [{title: "__T1__", tag: "__TAG1__"}, {title: "__T2__", tag: "__TAG2__"}]
  state doing = [{title: "__T3__", tag: "__TAG3__"}]
  state done = [{title: "__T4__", tag: "__TAG4__"}]
  computed todo-count = todo | count
  computed doing-count = doing | count
  computed done-count = done | count

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    row gap: 16px {
      column gap: 8px {
        row gap: 4px {
          rect width: 10px, height: 10px, color: __TODO_CLR__, radius: 5px
          text "To Do ({todo-count})" font-weight: bold, font-size: 14px
        }
        each card in todo {
          rect padding: 12px, color: __CARD_BG__, radius: 8px {
            text "{card.title}" font-size: 14px
            text "{card.tag}" color: #94a3b8, font-size: 11px
          }
        }
      }

      column gap: 8px {
        row gap: 4px {
          rect width: 10px, height: 10px, color: __DOING_CLR__, radius: 5px
          text "In Progress ({doing-count})" font-weight: bold, font-size: 14px
        }
        each card in doing {
          rect padding: 12px, color: __CARD_BG__, radius: 8px {
            text "{card.title}" font-size: 14px
            text "{card.tag}" color: #94a3b8, font-size: 11px
          }
        }
      }

      column gap: 8px {
        row gap: 4px {
          rect width: 10px, height: 10px, color: __DONE_CLR__, radius: 5px
          text "Done ({done-count})" font-weight: bold, font-size: 14px
        }
        each card in done {
          rect padding: 12px, color: __CARD_BG__, radius: 8px {
            text "{card.title}" font-size: 14px
            text "{card.tag}" color: #94a3b8, font-size: 11px
          }
        }
      }
    }
  }
}"""

for n, cfg in [
    ("sprint", {"TITLE": "Sprint Board", "DESC": "Agile sprint kanban board",
                "T1": "API endpoint", "TAG1": "backend", "T2": "Unit tests", "TAG2": "testing",
                "T3": "Auth flow", "TAG3": "frontend", "T4": "DB schema", "TAG4": "backend",
                "CLR": "#2563eb", "TODO_CLR": "#94a3b8", "DOING_CLR": "#3b82f6",
                "DONE_CLR": "#22c55e", "CARD_BG": "#f8fafc"}),
    ("design", {"TITLE": "Design Board", "DESC": "Design task tracking board",
                "T1": "Wireframes", "TAG1": "ux", "T2": "Icon set", "TAG2": "graphics",
                "T3": "Prototype", "TAG3": "ux", "T4": "Style guide", "TAG4": "brand",
                "CLR": "#ec4899", "TODO_CLR": "#d1d5db", "DOING_CLR": "#f472b6",
                "DONE_CLR": "#34d399", "CARD_BG": "#fdf2f8"}),
    ("marketing", {"TITLE": "Campaign Board", "DESC": "Marketing campaign task board",
                   "T1": "Blog post", "TAG1": "content", "T2": "Social graphics", "TAG2": "design",
                   "T3": "Email draft", "TAG3": "outreach", "T4": "Landing page", "TAG4": "web",
                   "CLR": "#f59e0b", "TODO_CLR": "#d1d5db", "DOING_CLR": "#fbbf24",
                   "DONE_CLR": "#4ade80", "CARD_BG": "#fffbeb"}),
    ("devops", {"TITLE": "DevOps Board", "DESC": "Infrastructure and deployment board",
                "T1": "Setup CI", "TAG1": "pipeline", "T2": "Monitoring", "TAG2": "observability",
                "T3": "Docker images", "TAG3": "containers", "T4": "SSL certs", "TAG4": "security",
                "CLR": "#475569", "TODO_CLR": "#9ca3af", "DOING_CLR": "#64748b",
                "DONE_CLR": "#16a34a", "CARD_BG": "#f1f5f9"}),
    ("bugfix", {"TITLE": "Bug Tracker", "DESC": "Bug tracking kanban board",
                "T1": "Login crash", "TAG1": "critical", "T2": "UI overlap", "TAG2": "cosmetic",
                "T3": "Memory leak", "TAG3": "performance", "T4": "Typo fix", "TAG4": "docs",
                "CLR": "#dc2626", "TODO_CLR": "#fca5a5", "DOING_CLR": "#ef4444",
                "DONE_CLR": "#22c55e", "CARD_BG": "#fef2f2"}),
    ("homework", {"TITLE": "Homework Board", "DESC": "Student homework tracking board",
                  "T1": "Math problems", "TAG1": "algebra", "T2": "Essay draft", "TAG2": "english",
                  "T3": "Lab report", "TAG3": "science", "T4": "Reading ch 5", "TAG4": "history",
                  "CLR": "#7c3aed", "TODO_CLR": "#c4b5fd", "DOING_CLR": "#8b5cf6",
                  "DONE_CLR": "#4ade80", "CARD_BG": "#f5f3ff"}),
    ("content", {"TITLE": "Content Board", "DESC": "Content creation pipeline board",
                 "T1": "Script outline", "TAG1": "video", "T2": "Thumbnail", "TAG2": "design",
                 "T3": "Editing", "TAG3": "video", "T4": "Published post", "TAG4": "blog",
                 "CLR": "#0891b2", "TODO_CLR": "#a5f3fc", "DOING_CLR": "#06b6d4",
                 "DONE_CLR": "#22c55e", "CARD_BG": "#ecfeff"}),
    ("renovation", {"TITLE": "Renovation Board", "DESC": "Home renovation project board",
                    "T1": "Get quotes", "TAG1": "planning", "T2": "Buy materials", "TAG2": "supplies",
                    "T3": "Paint walls", "TAG3": "labor", "T4": "Floor install", "TAG4": "labor",
                    "CLR": "#ea580c", "TODO_CLR": "#fdba74", "DOING_CLR": "#f97316",
                    "DONE_CLR": "#16a34a", "CARD_BG": "#fff7ed"}),
    ("launch", {"TITLE": "Launch Board", "DESC": "Product launch checklist board",
                "T1": "Press kit", "TAG1": "comms", "T2": "Demo video", "TAG2": "marketing",
                "T3": "Beta testing", "TAG3": "qa", "T4": "Docs site", "TAG4": "docs",
                "CLR": "#059669", "TODO_CLR": "#86efac", "DOING_CLR": "#10b981",
                "DONE_CLR": "#22c55e", "CARD_BG": "#ecfdf5"}),
    ("event", {"TITLE": "Event Board", "DESC": "Event planning task board",
               "T1": "Book venue", "TAG1": "logistics", "T2": "Invite speakers", "TAG2": "program",
               "T3": "Print badges", "TAG3": "materials", "T4": "Catering order", "TAG4": "food",
               "CLR": "#be185d", "TODO_CLR": "#f9a8d4", "DOING_CLR": "#ec4899",
               "DONE_CLR": "#34d399", "CARD_BG": "#fdf2f8"}),
]:
    ex(f"gen-board-{n}.naze", cfg["DESC"], fill(BOARD_T, cfg))


# ─── 8. Simulated charts using rects (gen-chart-*) — 8 examples ─────────────

ex("gen-chart-bar-sales.naze",
   "Bar chart showing monthly sales using colored rects",
   """-- Monthly sales bar chart
app "Sales Chart" {
  state jan = 120
  state feb = 180
  state mar = 150
  state apr = 210

  column padding: 24px, gap: 16px {
    heading "Monthly Sales" color: #1e40af

    row gap: 12px {
      column gap: 4px {
        rect width: 40px, height: 120px, color: #3b82f6, radius: 4px
        text "Jan" font-size: 12px, color: #64748b
        text "{jan}" font-size: 11px, color: #94a3b8
      }
      column gap: 4px {
        rect width: 40px, height: 180px, color: #3b82f6, radius: 4px
        text "Feb" font-size: 12px, color: #64748b
        text "{feb}" font-size: 11px, color: #94a3b8
      }
      column gap: 4px {
        rect width: 40px, height: 150px, color: #3b82f6, radius: 4px
        text "Mar" font-size: 12px, color: #64748b
        text "{mar}" font-size: 11px, color: #94a3b8
      }
      column gap: 4px {
        rect width: 40px, height: 210px, color: #3b82f6, radius: 4px
        text "Apr" font-size: 12px, color: #64748b
        text "{apr}" font-size: 11px, color: #94a3b8
      }
    }

    text "Values in thousands" color: #94a3b8, font-size: 12px
  }
}""")

ex("gen-chart-bar-revenue.naze",
   "Quarterly revenue bar chart with colored bars",
   """-- Quarterly revenue chart
app "Revenue Chart" {
  state q1 = 45
  state q2 = 62
  state q3 = 58
  state q4 = 80

  column padding: 24px, gap: 16px {
    heading "Revenue by Quarter" color: #15803d

    row gap: 16px {
      column gap: 4px {
        rect width: 50px, height: 90px, color: #22c55e, radius: 4px
        text "Q1" font-size: 13px
        text "${q1}K" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 50px, height: 124px, color: #16a34a, radius: 4px
        text "Q2" font-size: 13px
        text "${q2}K" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 50px, height: 116px, color: #22c55e, radius: 4px
        text "Q3" font-size: 13px
        text "${q3}K" color: #64748b, font-size: 12px
      }
      column gap: 4px {
        rect width: 50px, height: 160px, color: #15803d, radius: 4px
        text "Q4" font-size: 13px
        text "${q4}K" color: #64748b, font-size: 12px
      }
    }
  }
}""")

ex("gen-chart-horizontal.naze",
   "Horizontal bar chart showing skill levels",
   """-- Skill levels horizontal bars
app "Skill Chart" {
  column padding: 24px, gap: 12px {
    heading "Skill Levels" color: #4f46e5

    column gap: 8px {
      row gap: 8px {
        text "Python" font-size: 13px
        rect width: 200px, height: 20px, color: #6366f1, radius: 4px
      }
      row gap: 8px {
        text "Rust  " font-size: 13px
        rect width: 160px, height: 20px, color: #818cf8, radius: 4px
      }
      row gap: 8px {
        text "Go    " font-size: 13px
        rect width: 120px, height: 20px, color: #a5b4fc, radius: 4px
      }
      row gap: 8px {
        text "JS    " font-size: 13px
        rect width: 180px, height: 20px, color: #6366f1, radius: 4px
      }
      row gap: 8px {
        text "SQL   " font-size: 13px
        rect width: 140px, height: 20px, color: #818cf8, radius: 4px
      }
    }
  }
}""")

ex("gen-chart-dots.naze",
   "Dot plot showing data points as small circles",
   """-- Data points as dot plot
app "Usage Dots" {
  column padding: 24px, gap: 16px {
    heading "Weekly Usage" color: #0891b2

    text "Each dot = 10 users" color: #94a3b8, font-size: 12px

    column gap: 8px {
      row gap: 4px {
        text "Mon" font-size: 12px, color: #64748b
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
      }
      row gap: 4px {
        text "Tue" font-size: 12px, color: #64748b
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
      }
      row gap: 4px {
        text "Wed" font-size: 12px, color: #64748b
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
      }
      row gap: 4px {
        text "Thu" font-size: 12px, color: #64748b
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
      }
      row gap: 4px {
        text "Fri" font-size: 12px, color: #64748b
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
        rect width: 12px, height: 12px, color: #06b6d4, radius: 6px
      }
    }
  }
}""")

ex("gen-chart-stacked.naze",
   "Stacked bar chart for budget allocation",
   """-- Budget allocation stacked bars
app "Budget Allocation" {
  state engineering = 40
  state marketing = 25
  state operations = 20
  state other = 15

  column padding: 24px, gap: 16px {
    heading "Budget Split" color: #475569

    row gap: 0px {
      rect width: 160px, height: 32px, color: #3b82f6
      rect width: 100px, height: 32px, color: #10b981
      rect width: 80px, height: 32px, color: #f59e0b
      rect width: 60px, height: 32px, color: #94a3b8
    }

    grid columns: 2, gap: 8px {
      row gap: 4px {
        rect width: 12px, height: 12px, color: #3b82f6, radius: 2px
        text "Engineering {engineering}%" font-size: 13px
      }
      row gap: 4px {
        rect width: 12px, height: 12px, color: #10b981, radius: 2px
        text "Marketing {marketing}%" font-size: 13px
      }
      row gap: 4px {
        rect width: 12px, height: 12px, color: #f59e0b, radius: 2px
        text "Operations {operations}%" font-size: 13px
      }
      row gap: 4px {
        rect width: 12px, height: 12px, color: #94a3b8, radius: 2px
        text "Other {other}%" font-size: 13px
      }
    }
  }
}""")

ex("gen-chart-progress.naze",
   "Progress bars showing project completion percentages",
   """-- Project completion progress bars
app "Project Progress" {
  state alpha = 90
  state beta = 65
  state gamma = 30

  column padding: 24px, gap: 16px {
    heading "Project Status" color: #1e40af

    column gap: 12px {
      column gap: 4px {
        row gap: 8px {
          text "Alpha" font-weight: bold
          spacer
          text "{alpha}%" color: #16a34a
        }
        rect width: 300px, height: 8px, color: #e2e8f0, radius: 4px {
          rect width: 270px, height: 8px, color: #22c55e, radius: 4px
        }
      }

      column gap: 4px {
        row gap: 8px {
          text "Beta" font-weight: bold
          spacer
          text "{beta}%" color: #2563eb
        }
        rect width: 300px, height: 8px, color: #e2e8f0, radius: 4px {
          rect width: 195px, height: 8px, color: #3b82f6, radius: 4px
        }
      }

      column gap: 4px {
        row gap: 8px {
          text "Gamma" font-weight: bold
          spacer
          text "{gamma}%" color: #f59e0b
        }
        rect width: 300px, height: 8px, color: #e2e8f0, radius: 4px {
          rect width: 90px, height: 8px, color: #f59e0b, radius: 4px
        }
      }
    }
  }
}""")

ex("gen-chart-comparison.naze",
   "Side-by-side comparison bars for two products",
   """-- Product comparison bars
app "Feature Comparison" {
  column padding: 24px, gap: 16px {
    heading "Product Comparison" color: #475569

    row gap: 8px {
      rect width: 12px, height: 12px, color: #3b82f6, radius: 2px
      text "Product A" font-size: 13px
      rect width: 12px, height: 12px, color: #f97316, radius: 2px
      text "Product B" font-size: 13px
    }

    column gap: 8px {
      text "Performance" font-size: 13px, color: #64748b
      row gap: 4px {
        rect width: 180px, height: 16px, color: #3b82f6, radius: 2px
        rect width: 140px, height: 16px, color: #f97316, radius: 2px
      }

      text "Reliability" font-size: 13px, color: #64748b
      row gap: 4px {
        rect width: 160px, height: 16px, color: #3b82f6, radius: 2px
        rect width: 190px, height: 16px, color: #f97316, radius: 2px
      }

      text "Price Value" font-size: 13px, color: #64748b
      row gap: 4px {
        rect width: 120px, height: 16px, color: #3b82f6, radius: 2px
        rect width: 170px, height: 16px, color: #f97316, radius: 2px
      }
    }
  }
}""")

ex("gen-chart-heatmap.naze",
   "Simple heatmap grid showing activity intensity",
   """-- Activity heatmap grid
app "Activity Heatmap" {
  column padding: 24px, gap: 16px {
    heading "Weekly Activity" color: #15803d

    grid columns: 7, gap: 4px {
      rect width: 24px, height: 24px, color: #dcfce7, radius: 4px
      rect width: 24px, height: 24px, color: #86efac, radius: 4px
      rect width: 24px, height: 24px, color: #22c55e, radius: 4px
      rect width: 24px, height: 24px, color: #16a34a, radius: 4px
      rect width: 24px, height: 24px, color: #22c55e, radius: 4px
      rect width: 24px, height: 24px, color: #dcfce7, radius: 4px
      rect width: 24px, height: 24px, color: #f0fdf4, radius: 4px

      rect width: 24px, height: 24px, color: #86efac, radius: 4px
      rect width: 24px, height: 24px, color: #16a34a, radius: 4px
      rect width: 24px, height: 24px, color: #15803d, radius: 4px
      rect width: 24px, height: 24px, color: #16a34a, radius: 4px
      rect width: 24px, height: 24px, color: #86efac, radius: 4px
      rect width: 24px, height: 24px, color: #22c55e, radius: 4px
      rect width: 24px, height: 24px, color: #dcfce7, radius: 4px

      rect width: 24px, height: 24px, color: #f0fdf4, radius: 4px
      rect width: 24px, height: 24px, color: #dcfce7, radius: 4px
      rect width: 24px, height: 24px, color: #86efac, radius: 4px
      rect width: 24px, height: 24px, color: #22c55e, radius: 4px
      rect width: 24px, height: 24px, color: #16a34a, radius: 4px
      rect width: 24px, height: 24px, color: #22c55e, radius: 4px
      rect width: 24px, height: 24px, color: #86efac, radius: 4px
    }

    row gap: 4px {
      text "Less" color: #94a3b8, font-size: 11px
      rect width: 12px, height: 12px, color: #dcfce7, radius: 2px
      rect width: 12px, height: 12px, color: #86efac, radius: 2px
      rect width: 12px, height: 12px, color: #22c55e, radius: 2px
      rect width: 12px, height: 12px, color: #15803d, radius: 2px
      text "More" color: #94a3b8, font-size: 11px
    }
  }
}""")


# ─── 9. Onboarding flows (gen-onboard-*) — 10 examples ──────────────────────

ONBOARD_T = """-- __DESC__
app "__TITLE__" {
  state step = 1
  state name = ""
  state agreed = false

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    row gap: 8px {
      rect width: 32px, height: 32px, color: __CLR__, radius: 16px {
        text "1" color: #ffffff, font-size: 14px
      }
      rect width: 60px, height: 2px, color: #e2e8f0
      rect width: 32px, height: 32px, color: #e2e8f0, radius: 16px {
        text "2" font-size: 14px
      }
      rect width: 60px, height: 2px, color: #e2e8f0
      rect width: 32px, height: 32px, color: #e2e8f0, radius: 16px {
        text "3" font-size: 14px
      }
    }

    match step {
      1: column gap: 12px {
        text "__STEP1_TITLE__" font-size: 20px, font-weight: bold
        text "__STEP1_DESC__" color: #64748b
        rect width: 140px, height: 40px, color: __CLR__, radius: 8px {
          text "Get Started" color: #ffffff
          on click: set step = 2
        }
      }
      2: column gap: 12px {
        text "__STEP2_TITLE__" font-size: 20px, font-weight: bold
        text "__STEP2_DESC__" color: #64748b
        input bind: name, placeholder: "__PLACEHOLDER__"
        rect width: 100px, height: 40px, color: __CLR__, radius: 8px {
          text "Next" color: #ffffff
          on click: set step = 3
        }
      }
      3: column gap: 12px {
        text "__STEP3_TITLE__" font-size: 20px, font-weight: bold
        text "__STEP3_DESC__" color: #64748b
        checkbox bind: agreed, label: "__AGREE_LABEL__"
        rect width: 120px, height: 40px, color: __CLR__, radius: 8px {
          text "Complete" color: #ffffff
          on click: set step = 1
        }
      }
      _: text "Unknown step"
    }
  }
}"""

for n, cfg in [
    ("saas", {"TITLE": "SaaS Setup", "DESC": "SaaS product onboarding wizard",
              "STEP1_TITLE": "Welcome to CloudSync", "STEP1_DESC": "The easiest way to sync your data across devices.",
              "STEP2_TITLE": "Create Your Workspace", "STEP2_DESC": "Give your workspace a name to get started.",
              "STEP3_TITLE": "Terms and Conditions", "STEP3_DESC": "Please review and accept our terms.",
              "PLACEHOLDER": "Workspace name", "AGREE_LABEL": "I accept the terms",
              "CLR": "#2563eb"}),
    ("fitness-app", {"TITLE": "Fitness Onboard", "DESC": "Fitness app welcome flow",
                     "STEP1_TITLE": "Welcome to FitTrack", "STEP1_DESC": "Your personal fitness companion awaits.",
                     "STEP2_TITLE": "Set Your Goal", "STEP2_DESC": "What should we call you?",
                     "STEP3_TITLE": "Health Disclaimer", "STEP3_DESC": "Please acknowledge our health notice.",
                     "PLACEHOLDER": "Your name", "AGREE_LABEL": "I understand the health disclaimer",
                     "CLR": "#16a34a"}),
    ("social", {"TITLE": "Social Setup", "DESC": "Social media app onboarding",
                "STEP1_TITLE": "Join the Community", "STEP1_DESC": "Connect with people who share your interests.",
                "STEP2_TITLE": "Pick a Username", "STEP2_DESC": "Choose a unique handle for your profile.",
                "STEP3_TITLE": "Community Guidelines", "STEP3_DESC": "We believe in respectful conversations.",
                "PLACEHOLDER": "Username", "AGREE_LABEL": "I agree to the community guidelines",
                "CLR": "#8b5cf6"}),
    ("ecommerce", {"TITLE": "Shop Setup", "DESC": "E-commerce store setup wizard",
                   "STEP1_TITLE": "Open Your Store", "STEP1_DESC": "Start selling in minutes with our platform.",
                   "STEP2_TITLE": "Name Your Store", "STEP2_DESC": "Choose a memorable store name.",
                   "STEP3_TITLE": "Seller Agreement", "STEP3_DESC": "Review the seller terms and fees.",
                   "PLACEHOLDER": "Store name", "AGREE_LABEL": "I accept the seller agreement",
                   "CLR": "#ea580c"}),
    ("education", {"TITLE": "Course Setup", "DESC": "Online course enrollment onboarding",
                   "STEP1_TITLE": "Start Learning", "STEP1_DESC": "Thousands of courses at your fingertips.",
                   "STEP2_TITLE": "Your Details", "STEP2_DESC": "Tell us your display name.",
                   "STEP3_TITLE": "Academic Integrity", "STEP3_DESC": "We value honest learning.",
                   "PLACEHOLDER": "Display name", "AGREE_LABEL": "I pledge academic integrity",
                   "CLR": "#7c3aed"}),
    ("banking", {"TITLE": "Bank Onboard", "DESC": "Digital banking onboarding flow",
                 "STEP1_TITLE": "Welcome to NeoBank", "STEP1_DESC": "Banking made simple and transparent.",
                 "STEP2_TITLE": "Personal Information", "STEP2_DESC": "Enter your full legal name.",
                 "STEP3_TITLE": "Regulatory Agreement", "STEP3_DESC": "Required by financial regulations.",
                 "PLACEHOLDER": "Full name", "AGREE_LABEL": "I agree to the account terms",
                 "CLR": "#0369a1"}),
    ("devtool", {"TITLE": "Dev Tool Setup", "DESC": "Developer tool onboarding wizard",
                 "STEP1_TITLE": "Welcome, Developer", "STEP1_DESC": "Ship faster with our developer platform.",
                 "STEP2_TITLE": "Project Name", "STEP2_DESC": "What are you building today?",
                 "STEP3_TITLE": "API Terms of Use", "STEP3_DESC": "Review our API usage policy.",
                 "PLACEHOLDER": "Project name", "AGREE_LABEL": "I accept the API terms",
                 "CLR": "#475569"}),
    ("gaming", {"TITLE": "Game Setup", "DESC": "Gaming platform onboarding",
                "STEP1_TITLE": "Ready to Play", "STEP1_DESC": "Join millions of gamers worldwide.",
                "STEP2_TITLE": "Choose Your Tag", "STEP2_DESC": "Pick a gamer tag that represents you.",
                "STEP3_TITLE": "Fair Play Policy", "STEP3_DESC": "We enforce a strict no-cheating policy.",
                "PLACEHOLDER": "Gamer tag", "AGREE_LABEL": "I agree to play fair",
                "CLR": "#9333ea"}),
    ("travel", {"TITLE": "Travel App Setup", "DESC": "Travel booking app onboarding",
                "STEP1_TITLE": "Explore the World", "STEP1_DESC": "Book flights, hotels, and adventures.",
                "STEP2_TITLE": "Traveler Profile", "STEP2_DESC": "How should we address you?",
                "STEP3_TITLE": "Booking Terms", "STEP3_DESC": "Review cancellation and refund policies.",
                "PLACEHOLDER": "Preferred name", "AGREE_LABEL": "I accept the booking terms",
                "CLR": "#0891b2"}),
    ("newsletter", {"TITLE": "Newsletter Setup", "DESC": "Newsletter subscription onboarding",
                    "STEP1_TITLE": "Stay Informed", "STEP1_DESC": "Get curated content delivered weekly.",
                    "STEP2_TITLE": "Your Email", "STEP2_DESC": "Enter the name you want us to use.",
                    "STEP3_TITLE": "Privacy Notice", "STEP3_DESC": "We respect your inbox and privacy.",
                    "PLACEHOLDER": "First name", "AGREE_LABEL": "I consent to receiving emails",
                    "CLR": "#be185d"}),
]:
    ex(f"gen-onboard-{n}.naze", cfg["DESC"], fill(ONBOARD_T, cfg))


# ─── 10. Notification centers (gen-notif-*) — 10 examples ───────────────────

NOTIF_T = """-- __DESC__
app "__TITLE__" {
  state unread = __UNREAD__
  state notifications = [__ITEMS__]
  computed total = notifications | count
  state filter = "all"

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__

    row gap: 8px {
      text "Notifications" font-size: 18px, font-weight: bold
      rect padding: 4px, color: __BADGE_BG__, radius: 10px {
        text "{unread}" color: __BADGE_CLR__, font-size: 12px
      }
      spacer
      rect padding: 8px, color: #f1f5f9, radius: 4px {
        text "Mark all read" font-size: 12px, color: #64748b
        on click: set unread = 0
      }
    }

    row gap: 8px {
      rect padding: 6px, color: __CLR__, radius: 4px {
        text "All" color: #ffffff, font-size: 12px
        on click: set filter = "all"
      }
      rect padding: 6px, color: #e2e8f0, radius: 4px {
        text "__CAT1__" font-size: 12px
        on click: set filter = "__CAT1__"
      }
      rect padding: 6px, color: #e2e8f0, radius: 4px {
        text "__CAT2__" font-size: 12px
        on click: set filter = "__CAT2__"
      }
    }

    text "{total} notifications" color: #94a3b8, font-size: 12px

    each notif in notifications {
      rect padding: 12px, color: __ITEM_BG__, radius: 8px {
        column gap: 4px {
          row gap: 8px {
            rect width: 8px, height: 8px, color: __DOT_CLR__, radius: 4px
            text "{notif.title}" font-weight: bold, font-size: 14px
          }
          text "{notif.body}" color: #64748b, font-size: 13px
          text "{notif.time}" color: #94a3b8, font-size: 11px
        }
      }
    }
  }
}"""

for n, cfg in [
    ("inbox", {"TITLE": "Inbox Alerts", "DESC": "Email inbox notification center",
               "UNREAD": "5",
               "ITEMS": '{title: "New message", body: "Alice sent you a message", time: "2 min ago"}, {title: "Reply received", body: "Bob replied to your thread", time: "15 min ago"}, {title: "Mention", body: "You were mentioned in a comment", time: "1 hour ago"}',
               "CAT1": "Messages", "CAT2": "Mentions",
               "CLR": "#2563eb", "BADGE_BG": "#dbeafe", "BADGE_CLR": "#1d4ed8",
               "DOT_CLR": "#3b82f6", "ITEM_BG": "#f8fafc"}),
    ("github", {"TITLE": "Repo Alerts", "DESC": "GitHub-style repository notifications",
                "UNREAD": "8",
                "ITEMS": '{title: "PR Approved", body: "Your pull request was approved", time: "5 min ago"}, {title: "Issue Assigned", body: "Bug #142 assigned to you", time: "30 min ago"}, {title: "CI Passed", body: "Build pipeline succeeded", time: "1 hour ago"}',
                "CAT1": "PRs", "CAT2": "Issues",
                "CLR": "#475569", "BADGE_BG": "#f1f5f9", "BADGE_CLR": "#334155",
                "DOT_CLR": "#64748b", "ITEM_BG": "#f8fafc"}),
    ("social-feed", {"TITLE": "Social Alerts", "DESC": "Social media notification feed",
                     "UNREAD": "12",
                     "ITEMS": '{title: "New follower", body: "Emma started following you", time: "Just now"}, {title: "Like", body: "Your post got 50 likes", time: "10 min ago"}, {title: "Comment", body: "Jake commented on your photo", time: "1 hour ago"}',
                     "CAT1": "Likes", "CAT2": "Comments",
                     "CLR": "#ec4899", "BADGE_BG": "#fce7f3", "BADGE_CLR": "#be185d",
                     "DOT_CLR": "#f472b6", "ITEM_BG": "#fdf2f8"}),
    ("ecommerce-alerts", {"TITLE": "Shop Alerts", "DESC": "E-commerce order notifications",
                          "UNREAD": "3",
                          "ITEMS": '{title: "Order shipped", body: "Your package is on the way", time: "1 hour ago"}, {title: "Price drop", body: "Wishlist item now 20% off", time: "3 hours ago"}, {title: "Review request", body: "Rate your recent purchase", time: "Yesterday"}',
                          "CAT1": "Orders", "CAT2": "Deals",
                          "CLR": "#ea580c", "BADGE_BG": "#fff7ed", "BADGE_CLR": "#c2410c",
                          "DOT_CLR": "#f97316", "ITEM_BG": "#fffbeb"}),
    ("system", {"TITLE": "System Alerts", "DESC": "System monitoring notifications",
                "UNREAD": "2",
                "ITEMS": '{title: "CPU Alert", body: "CPU usage exceeded 90%", time: "5 min ago"}, {title: "Disk Warning", body: "Disk space below 10%", time: "1 hour ago"}, {title: "Update Available", body: "System update v2.5 ready", time: "Today"}',
                "CAT1": "Warnings", "CAT2": "Updates",
                "CLR": "#dc2626", "BADGE_BG": "#fef2f2", "BADGE_CLR": "#b91c1c",
                "DOT_CLR": "#ef4444", "ITEM_BG": "#fef2f2"}),
    ("team", {"TITLE": "Team Updates", "DESC": "Team collaboration notifications",
              "UNREAD": "6",
              "ITEMS": '{title: "Meeting invite", body: "Sprint planning at 2 PM", time: "10 min ago"}, {title: "Doc shared", body: "Sarah shared a design spec", time: "30 min ago"}, {title: "Task completed", body: "Mike finished the API docs", time: "2 hours ago"}',
              "CAT1": "Meetings", "CAT2": "Tasks",
              "CLR": "#7c3aed", "BADGE_BG": "#f5f3ff", "BADGE_CLR": "#6d28d9",
              "DOT_CLR": "#8b5cf6", "ITEM_BG": "#faf5ff"}),
    ("banking-alerts", {"TITLE": "Bank Alerts", "DESC": "Banking transaction notifications",
                        "UNREAD": "4",
                        "ITEMS": '{title: "Payment received", body: "$250 deposit from PayCo", time: "Just now"}, {title: "Card charge", body: "$42.50 at Coffee Shop", time: "2 hours ago"}, {title: "Bill due", body: "Electric bill due in 3 days", time: "Today"}',
                        "CAT1": "Payments", "CAT2": "Bills",
                        "CLR": "#059669", "BADGE_BG": "#ecfdf5", "BADGE_CLR": "#047857",
                        "DOT_CLR": "#10b981", "ITEM_BG": "#f0fdf4"}),
    ("health", {"TITLE": "Health Alerts", "DESC": "Health and wellness notifications",
                "UNREAD": "3",
                "ITEMS": '{title: "Step goal reached", body: "You hit 10K steps today", time: "30 min ago"}, {title: "Hydration reminder", body: "Time to drink some water", time: "1 hour ago"}, {title: "Sleep report", body: "You slept 7.5 hours last night", time: "This morning"}',
                "CAT1": "Activity", "CAT2": "Reminders",
                "CLR": "#0891b2", "BADGE_BG": "#ecfeff", "BADGE_CLR": "#0e7490",
                "DOT_CLR": "#06b6d4", "ITEM_BG": "#f0f9ff"}),
    ("school", {"TITLE": "School Alerts", "DESC": "Student school notifications",
                "UNREAD": "7",
                "ITEMS": '{title: "Grade posted", body: "Math exam: 92/100", time: "1 hour ago"}, {title: "Assignment due", body: "Essay due tomorrow at noon", time: "3 hours ago"}, {title: "Class cancelled", body: "Physics lab moved to Friday", time: "Today"}',
                "CAT1": "Grades", "CAT2": "Classes",
                "CLR": "#4f46e5", "BADGE_BG": "#eef2ff", "BADGE_CLR": "#4338ca",
                "DOT_CLR": "#6366f1", "ITEM_BG": "#eef2ff"}),
    ("delivery", {"TITLE": "Delivery Alerts", "DESC": "Food delivery order notifications",
                  "UNREAD": "2",
                  "ITEMS": '{title: "Order confirmed", body: "Your order is being prepared", time: "5 min ago"}, {title: "Driver assigned", body: "Marco is picking up your food", time: "10 min ago"}, {title: "Arriving soon", body: "Your delivery is 3 min away", time: "Just now"}',
                  "CAT1": "Active", "CAT2": "Past",
                  "CLR": "#15803d", "BADGE_BG": "#dcfce7", "BADGE_CLR": "#166534",
                  "DOT_CLR": "#22c55e", "ITEM_BG": "#f0fdf4"}),
]:
    ex(f"gen-notif-{n}.naze", cfg["DESC"], fill(NOTIF_T, cfg))


# ═══════════════════════════════════════════════════════════════════════════════
# BATCH B: 100 additional training examples (10 categories x 10 each)
# ═══════════════════════════════════════════════════════════════════════════════

# ─── 1. Booking UIs (10) ─────────────────────────────────────────────────────

BOOKING_T = """-- __DESC__
app "__TITLE__" {
  state guest-name = ""
  state __F2__ = ""
  state __F3__ = __F3V__
  state confirmed = false

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__
    text "__SUBTITLE__" color: #64748b

    input bind: guest-name, placeholder: "Your name"
    input bind: __F2__, placeholder: "__P2__"
    input bind: __F3__, placeholder: "__P3__"

    rect width: 140px, height: 44px, color: __CLR__, radius: 8px {
      text "__BTN__" color: #ffffff
      on click: set confirmed = true
    }

    if confirmed {
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "Confirmed for {guest-name}" color: #16a34a, font-weight: bold
        text "__CONF_MSG__" color: #64748b
      }
    }
  }
}"""

for n, cfg in [
    ("hotel", {"TITLE": "Hotel Booking", "DESC": "Hotel room reservation form",
     "SUBTITLE": "Reserve your stay", "F2": "check-in", "P2": "Check-in date",
     "F3": "nights", "F3V": '""', "P3": "Number of nights",
     "BTN": "Reserve", "CLR": "#2563eb", "CONF_MSG": "Your room is booked!"}),
    ("restaurant", {"TITLE": "Table Reservation", "DESC": "Restaurant table booking",
     "SUBTITLE": "Book a table", "F2": "party-size", "P2": "Party size",
     "F3": "time-slot", "F3V": '""', "P3": "Preferred time",
     "BTN": "Book Table", "CLR": "#16a34a", "CONF_MSG": "Table reserved!"}),
    ("appointment", {"TITLE": "Appointment", "DESC": "Doctor appointment scheduler",
     "SUBTITLE": "Schedule your visit", "F2": "appt-date", "P2": "Preferred date",
     "F3": "reason", "F3V": '""', "P3": "Reason for visit",
     "BTN": "Schedule", "CLR": "#8b5cf6", "CONF_MSG": "Appointment scheduled!"}),
    ("spa", {"TITLE": "Spa Booking", "DESC": "Spa treatment reservation",
     "SUBTITLE": "Book a treatment", "F2": "treatment", "P2": "Treatment type",
     "F3": "duration", "F3V": '""', "P3": "Duration (minutes)",
     "BTN": "Book Spa", "CLR": "#ec4899", "CONF_MSG": "Spa session booked!"}),
    ("flight", {"TITLE": "Flight Booking", "DESC": "Flight reservation form",
     "SUBTITLE": "Book your flight", "F2": "destination", "P2": "Destination city",
     "F3": "passengers", "F3V": '""', "P3": "Number of passengers",
     "BTN": "Book Flight", "CLR": "#0ea5e9", "CONF_MSG": "Flight booked!"}),
    ("tour", {"TITLE": "Tour Booking", "DESC": "Guided tour reservation",
     "SUBTITLE": "Reserve a tour", "F2": "tour-date", "P2": "Tour date",
     "F3": "group-size", "F3V": '""', "P3": "Group size",
     "BTN": "Reserve Tour", "CLR": "#f59e0b", "CONF_MSG": "Tour reserved!"}),
    ("car-rental", {"TITLE": "Car Rental", "DESC": "Vehicle rental booking form",
     "SUBTITLE": "Rent a vehicle", "F2": "pickup-date", "P2": "Pickup date",
     "F3": "car-type", "F3V": '""', "P3": "Vehicle type",
     "BTN": "Rent Now", "CLR": "#6366f1", "CONF_MSG": "Rental confirmed!"}),
    ("salon", {"TITLE": "Salon Appointment", "DESC": "Hair salon booking",
     "SUBTITLE": "Book a haircut", "F2": "service", "P2": "Service type",
     "F3": "stylist-pref", "F3V": '""', "P3": "Preferred stylist",
     "BTN": "Book Salon", "CLR": "#d946ef", "CONF_MSG": "Salon appointment set!"}),
]:
    ex(f"gen-booking-{n}.naze", cfg["DESC"], fill(BOOKING_T, cfg))

# Two hand-crafted booking examples with richer UI

ex("gen-booking-venue.naze",
   "Event venue reservation with capacity and date selection",
   """-- Venue reservation with event details
app "Venue Booking" {
  state venue-name = ""
  state event-date = ""
  state attendees = ""
  state event-type = "conference"
  state booked = false

  column padding: 24px, gap: 16px {
    heading "Reserve a Venue" color: #1e40af
    text "Find the perfect space for your event" color: #64748b

    input bind: venue-name, placeholder: "Venue or hall name"
    input bind: event-date, placeholder: "Event date"
    input bind: attendees, placeholder: "Expected attendees"

    select bind: event-type {
      option "Conference" value: "conference"
      option "Wedding" value: "wedding"
      option "Workshop" value: "workshop"
      option "Party" value: "party"
    }

    rect width: 160px, height: 44px, color: #1e40af, radius: 8px {
      text "Reserve Venue" color: #ffffff
      on click: set booked = true
    }

    if booked {
      rect padding: 16px, color: #eff6ff, radius: 8px {
        text "Venue reserved!" color: #1e40af, font-weight: bold
        text "Type: {event-type}" color: #64748b
      }
    }
  }
}""")

ex("gen-booking-class.naze",
   "Fitness class booking with time slots and instructor info",
   """-- Fitness class booking
app "Class Booking" {
  state student = ""
  state class-time = ""
  state signed-up = false

  state classes = [{name: "Yoga", instructor: "Sarah", spots: "5"}, {name: "Pilates", instructor: "Mike", spots: "3"}, {name: "HIIT", instructor: "Jess", spots: "8"}]

  column padding: 24px, gap: 16px {
    heading "Book a Class" color: #7c3aed

    each cls in classes {
      row padding: 12px, color: #f5f3ff, radius: 8px, gap: 12px {
        text "{cls.name}" font-weight: bold
        text "with {cls.instructor}" color: #64748b
        text "{cls.spots} spots" color: #7c3aed
      }
    }

    separator

    input bind: student, placeholder: "Your name"
    input bind: class-time, placeholder: "Preferred time"

    rect width: 120px, height: 40px, color: #7c3aed, radius: 8px {
      text "Sign Up" color: #ffffff
      on click: set signed-up = true
    }

    if signed-up {
      text "You are signed up, {student}!" color: #16a34a
    }
  }
}""")

# ─── 2. Voting Polls & Surveys (10) ──────────────────────────────────────────

POLL_T = """-- __DESC__
app "__TITLE__" {
  state opt-a = __VA__
  state opt-b = __VB__
  state opt-c = __VC__
  computed total-votes = opt-a + opt-b + opt-c

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__
    text "__QUESTION__" font-size: 18px
    text "Total votes: {total-votes}" color: #64748b

    row gap: 8px {
      rect width: 120px, height: 40px, color: __CLR__, radius: 8px {
        text "__LA__ ({opt-a})" color: #ffffff
        on click: set opt-a = opt-a + 1
      }
      rect width: 120px, height: 40px, color: #64748b, radius: 8px {
        text "__LB__ ({opt-b})" color: #ffffff
        on click: set opt-b = opt-b + 1
      }
      rect width: 120px, height: 40px, color: #94a3b8, radius: 8px {
        text "__LC__ ({opt-c})" color: #ffffff
        on click: set opt-c = opt-c + 1
      }
    }

    column gap: 4px {
      rect width: 300px, height: 24px, color: #e2e8f0, radius: 4px {
        text "__LA__" font-size: 12px
      }
      rect width: 300px, height: 24px, color: #e2e8f0, radius: 4px {
        text "__LB__" font-size: 12px
      }
      rect width: 300px, height: 24px, color: #e2e8f0, radius: 4px {
        text "__LC__" font-size: 12px
      }
    }
  }
}"""

for n, cfg in [
    ("language", {"TITLE": "Favorite Language", "DESC": "Programming language preference poll",
     "QUESTION": "Which language do you prefer?", "VA": 12, "VB": 8, "VC": 5,
     "LA": "Rust", "LB": "Python", "LC": "Go", "CLR": "#f97316"}),
    ("food", {"TITLE": "Food Poll", "DESC": "Favorite cuisine voting poll",
     "QUESTION": "Best cuisine?", "VA": 20, "VB": 15, "VC": 10,
     "LA": "Italian", "LB": "Japanese", "LC": "Mexican", "CLR": "#ef4444"}),
    ("framework", {"TITLE": "Framework Poll", "DESC": "Frontend framework preference survey",
     "QUESTION": "Preferred framework?", "VA": 30, "VB": 25, "VC": 18,
     "LA": "React", "LB": "Vue", "LC": "Svelte", "CLR": "#3b82f6"}),
    ("music", {"TITLE": "Music Genre Poll", "DESC": "Music genre popularity survey",
     "QUESTION": "Favorite genre?", "VA": 15, "VB": 12, "VC": 9,
     "LA": "Rock", "LB": "Jazz", "LC": "Electronic", "CLR": "#8b5cf6"}),
    ("color", {"TITLE": "Color Poll", "DESC": "Favorite color voting",
     "QUESTION": "Pick your favorite color", "VA": 22, "VB": 18, "VC": 14,
     "LA": "Blue", "LB": "Green", "LC": "Red", "CLR": "#2563eb"}),
    ("os", {"TITLE": "OS Preference", "DESC": "Operating system preference poll",
     "QUESTION": "Which OS do you use?", "VA": 35, "VB": 28, "VC": 20,
     "LA": "Linux", "LB": "macOS", "LC": "Windows", "CLR": "#16a34a"}),
    ("season", {"TITLE": "Season Poll", "DESC": "Favorite season voting poll",
     "QUESTION": "Best season of the year?", "VA": 18, "VB": 22, "VC": 15,
     "LA": "Summer", "LB": "Autumn", "LC": "Spring", "CLR": "#f59e0b"}),
    ("pet", {"TITLE": "Pet Poll", "DESC": "Favorite pet type survey",
     "QUESTION": "Cats or dogs?", "VA": 40, "VB": 38, "VC": 12,
     "LA": "Dogs", "LB": "Cats", "LC": "Birds", "CLR": "#ec4899"}),
]:
    ex(f"gen-poll-{n}.naze", cfg["DESC"], fill(POLL_T, cfg))

# Two hand-crafted poll examples

ex("gen-poll-satisfaction.naze",
   "Customer satisfaction survey with emoji-style ratings",
   """-- Customer satisfaction poll
app "Satisfaction Survey" {
  state great = 0
  state okay = 0
  state poor = 0
  state submitted = false
  computed responses = great + okay + poor

  column padding: 24px, gap: 16px {
    heading "How was your experience?" color: #1e293b
    text "{responses} responses so far" color: #64748b

    row gap: 16px {
      column gap: 4px {
        rect width: 80px, height: 80px, color: #dcfce7, radius: 12px {
          text "Great" color: #16a34a, font-size: 16px
          on click: set great = great + 1
        }
        text "{great}" color: #16a34a
      }
      column gap: 4px {
        rect width: 80px, height: 80px, color: #fef9c3, radius: 12px {
          text "Okay" color: #a16207, font-size: 16px
          on click: set okay = okay + 1
        }
        text "{okay}" color: #a16207
      }
      column gap: 4px {
        rect width: 80px, height: 80px, color: #fee2e2, radius: 12px {
          text "Poor" color: #dc2626, font-size: 16px
          on click: set poor = poor + 1
        }
        text "{poor}" color: #dc2626
      }
    }
  }
}""")

ex("gen-poll-binary.naze",
   "Simple yes/no poll with live vote count",
   """-- Yes/No binary poll
app "Quick Poll" {
  state yes-count = 0
  state no-count = 0
  computed total = yes-count + no-count

  column padding: 24px, gap: 16px {
    heading "Do you like declarative UI?" color: #0f172a

    row gap: 16px {
      rect width: 120px, height: 50px, color: #16a34a, radius: 8px {
        text "Yes ({yes-count})" color: #ffffff, font-size: 18px
        on click: set yes-count = yes-count + 1
      }
      rect width: 120px, height: 50px, color: #dc2626, radius: 8px {
        text "No ({no-count})" color: #ffffff, font-size: 18px
        on click: set no-count = no-count + 1
      }
    }

    text "{total} total votes" color: #64748b
  }
}""")

# ─── 3. FAQ Pages (10) ───────────────────────────────────────────────────────

FAQ_T = """-- __DESC__
app "__TITLE__" {
  state show-q1 = false
  state show-q2 = false
  state show-q3 = false

  column padding: 24px, gap: 12px {
    heading "__TITLE__" color: __CLR__
    text "__SUBTITLE__" color: #64748b

    rect padding: 12px, color: #f8fafc, radius: 8px {
      text "__Q1__" font-weight: bold
      on click: set show-q1 = true
    }
    if show-q1 {
      text "__A1__" color: #475569
    }

    rect padding: 12px, color: #f8fafc, radius: 8px {
      text "__Q2__" font-weight: bold
      on click: set show-q2 = true
    }
    if show-q2 {
      text "__A2__" color: #475569
    }

    rect padding: 12px, color: #f8fafc, radius: 8px {
      text "__Q3__" font-weight: bold
      on click: set show-q3 = true
    }
    if show-q3 {
      text "__A3__" color: #475569
    }
  }
}"""

for n, cfg in [
    ("shipping", {"TITLE": "Shipping FAQ", "DESC": "Shipping frequently asked questions",
     "SUBTITLE": "Common shipping questions", "CLR": "#2563eb",
     "Q1": "How long does shipping take?", "A1": "Standard shipping takes 5-7 business days.",
     "Q2": "Do you ship internationally?", "A2": "Yes, we ship to over 50 countries.",
     "Q3": "Can I track my order?", "A3": "Yes, a tracking number is emailed after dispatch."}),
    ("returns", {"TITLE": "Returns FAQ", "DESC": "Product return policy questions",
     "SUBTITLE": "Return policy details", "CLR": "#dc2626",
     "Q1": "What is the return window?", "A1": "You have 30 days to return unused items.",
     "Q2": "How do I start a return?", "A2": "Contact support with your order number.",
     "Q3": "Who pays for return shipping?", "A3": "We provide a prepaid return label."}),
    ("billing", {"TITLE": "Billing FAQ", "DESC": "Billing and payment questions",
     "SUBTITLE": "Payment-related questions", "CLR": "#16a34a",
     "Q1": "What payment methods do you accept?", "A1": "We accept Visa, Mastercard, and PayPal.",
     "Q2": "Can I change my plan?", "A2": "Yes, upgrade or downgrade anytime from settings.",
     "Q3": "When am I billed?", "A3": "Billing occurs on the 1st of each month."}),
    ("account", {"TITLE": "Account FAQ", "DESC": "User account management questions",
     "SUBTITLE": "Account help", "CLR": "#8b5cf6",
     "Q1": "How do I reset my password?", "A1": "Click Forgot Password on the login page.",
     "Q2": "Can I change my username?", "A2": "Yes, go to Settings then Profile.",
     "Q3": "How do I delete my account?", "A3": "Contact support to request deletion."}),
    ("privacy", {"TITLE": "Privacy FAQ", "DESC": "Data privacy information page",
     "SUBTITLE": "How we protect your data", "CLR": "#0f172a",
     "Q1": "What data do you collect?", "A1": "Only email and usage analytics.",
     "Q2": "Do you sell user data?", "A2": "No. We never sell personal data.",
     "Q3": "How can I export my data?", "A3": "Use the Data Export option in settings."}),
    ("technical", {"TITLE": "Technical FAQ", "DESC": "Technical support questions",
     "SUBTITLE": "Tech support answers", "CLR": "#f97316",
     "Q1": "What browsers are supported?", "A1": "Chrome, Firefox, Safari, and Edge.",
     "Q2": "Is there a mobile app?", "A2": "Yes, available on iOS and Android.",
     "Q3": "What is the uptime guarantee?", "A3": "We guarantee 99.9% uptime."}),
    ("onboarding", {"TITLE": "Getting Started FAQ", "DESC": "Onboarding help questions",
     "SUBTITLE": "New user guide", "CLR": "#6366f1",
     "Q1": "How do I create an account?", "A1": "Click Sign Up and enter your email.",
     "Q2": "Is there a free trial?", "A2": "Yes, 14 days free with full access.",
     "Q3": "Where is the documentation?", "A3": "Visit docs.example.com for guides."}),
    ("subscription", {"TITLE": "Subscription FAQ", "DESC": "Subscription plan questions",
     "SUBTITLE": "Plan details", "CLR": "#ec4899",
     "Q1": "Can I cancel anytime?", "A1": "Yes, cancel with no fees from your dashboard.",
     "Q2": "Is there a student discount?", "A2": "Yes, students get 50% off.",
     "Q3": "Do you offer annual billing?", "A3": "Yes, save 20% with annual plans."}),
]:
    ex(f"gen-faq-{n}.naze", cfg["DESC"], fill(FAQ_T, cfg))

# Two hand-crafted FAQ examples

ex("gen-faq-product.naze",
   "Product FAQ with expandable answers and category list",
   """-- Product FAQ with categories
app "Product FAQ" {
  state show-size = false
  state show-material = false
  state show-care = false
  state show-warranty = false

  column padding: 24px, gap: 12px {
    heading "Product Questions" color: #1e293b
    text "Everything about our products" color: #64748b
    separator

    rect padding: 14px, color: #fef3c7, radius: 8px {
      text "What sizes are available?" font-weight: bold
      on click: set show-size = true
    }
    if show-size {
      text "We offer XS through XXL in all styles." color: #475569
    }

    rect padding: 14px, color: #fef3c7, radius: 8px {
      text "What materials do you use?" font-weight: bold
      on click: set show-material = true
    }
    if show-material {
      text "100% organic cotton and recycled polyester." color: #475569
    }

    rect padding: 14px, color: #fef3c7, radius: 8px {
      text "How do I care for the product?" font-weight: bold
      on click: set show-care = true
    }
    if show-care {
      text "Machine wash cold, tumble dry low." color: #475569
    }

    rect padding: 14px, color: #fef3c7, radius: 8px {
      text "Is there a warranty?" font-weight: bold
      on click: set show-warranty = true
    }
    if show-warranty {
      text "All products carry a 1-year warranty." color: #475569
    }
  }
}""")

ex("gen-faq-api.naze",
   "API documentation FAQ with developer-focused questions",
   """-- API documentation FAQ
app "API FAQ" {
  state show-auth = false
  state show-limits = false
  state show-format = false

  column padding: 24px, gap: 12px {
    heading "API FAQ" color: #0ea5e9
    text "Developer questions answered" color: #64748b

    rect padding: 12px, color: #f0f9ff, radius: 8px {
      text "How do I authenticate?" font-weight: bold
      on click: set show-auth = true
    }
    if show-auth {
      text "Use Bearer token in the Authorization header." color: #475569
    }

    rect padding: 12px, color: #f0f9ff, radius: 8px {
      text "What are the rate limits?" font-weight: bold
      on click: set show-limits = true
    }
    if show-limits {
      text "1000 requests per hour for free tier." color: #475569
    }

    rect padding: 12px, color: #f0f9ff, radius: 8px {
      text "What response format is used?" font-weight: bold
      on click: set show-format = true
    }
    if show-format {
      text "All responses are JSON with UTF-8 encoding." color: #475569
    }
  }
}""")

# ─── 4. Pricing Tables (10) ──────────────────────────────────────────────────

PRICING_T = """-- __DESC__
app "__TITLE__" {
  state selected = "none"

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: #0f172a
    text "__SUBTITLE__" color: #64748b

    grid columns: 3, gap: 16px {
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "__P1_NAME__" font-weight: bold, font-size: 18px
          text "__P1_PRICE__" color: __CLR__, font-size: 28px
          text "__P1_DESC__" color: #64748b, font-size: 14px
          separator
          text "__P1_F1__" font-size: 14px
          text "__P1_F2__" font-size: 14px
          rect width: 120px, height: 36px, color: #e2e8f0, radius: 6px {
            text "Choose" color: #475569
            on click: set selected = "__P1_NAME__"
          }
        }
      }
      rect padding: 20px, color: __FEAT_BG__, radius: 12px {
        column gap: 8px {
          text "__P2_NAME__" font-weight: bold, font-size: 18px
          text "__P2_PRICE__" color: __CLR__, font-size: 28px
          text "__P2_DESC__" color: #64748b, font-size: 14px
          separator
          text "__P2_F1__" font-size: 14px
          text "__P2_F2__" font-size: 14px
          rect width: 120px, height: 36px, color: __CLR__, radius: 6px {
            text "Choose" color: #ffffff
            on click: set selected = "__P2_NAME__"
          }
        }
      }
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "__P3_NAME__" font-weight: bold, font-size: 18px
          text "__P3_PRICE__" color: __CLR__, font-size: 28px
          text "__P3_DESC__" color: #64748b, font-size: 14px
          separator
          text "__P3_F1__" font-size: 14px
          text "__P3_F2__" font-size: 14px
          rect width: 120px, height: 36px, color: #e2e8f0, radius: 6px {
            text "Choose" color: #475569
            on click: set selected = "__P3_NAME__"
          }
        }
      }
    }

    if selected != "none" {
      text "Selected: {selected}" color: __CLR__, font-weight: bold
    }
  }
}"""

for n, cfg in [
    ("saas", {"TITLE": "SaaS Pricing", "DESC": "SaaS subscription pricing table",
     "SUBTITLE": "Choose the plan that fits your needs", "CLR": "#2563eb", "FEAT_BG": "#eff6ff",
     "P1_NAME": "Starter", "P1_PRICE": "$9/mo", "P1_DESC": "For individuals",
     "P1_F1": "5 projects", "P1_F2": "1GB storage",
     "P2_NAME": "Pro", "P2_PRICE": "$29/mo", "P2_DESC": "For teams",
     "P2_F1": "Unlimited projects", "P2_F2": "50GB storage",
     "P3_NAME": "Enterprise", "P3_PRICE": "$99/mo", "P3_DESC": "For organizations",
     "P3_F1": "Custom limits", "P3_F2": "Priority support"}),
    ("hosting", {"TITLE": "Hosting Plans", "DESC": "Web hosting pricing comparison",
     "SUBTITLE": "Reliable hosting for every site", "CLR": "#16a34a", "FEAT_BG": "#f0fdf4",
     "P1_NAME": "Basic", "P1_PRICE": "$5/mo", "P1_DESC": "Shared hosting",
     "P1_F1": "10GB disk", "P1_F2": "1 domain",
     "P2_NAME": "Business", "P2_PRICE": "$15/mo", "P2_DESC": "VPS hosting",
     "P2_F1": "50GB SSD", "P2_F2": "5 domains",
     "P3_NAME": "Dedicated", "P3_PRICE": "$49/mo", "P3_DESC": "Bare metal",
     "P3_F1": "500GB NVMe", "P3_F2": "Unlimited domains"}),
    ("api", {"TITLE": "API Pricing", "DESC": "API access tier pricing",
     "SUBTITLE": "Scale as you grow", "CLR": "#f97316", "FEAT_BG": "#fff7ed",
     "P1_NAME": "Free", "P1_PRICE": "$0/mo", "P1_DESC": "Getting started",
     "P1_F1": "1K calls/day", "P1_F2": "Community support",
     "P2_NAME": "Growth", "P2_PRICE": "$49/mo", "P2_DESC": "Production use",
     "P2_F1": "100K calls/day", "P2_F2": "Email support",
     "P3_NAME": "Scale", "P3_PRICE": "$199/mo", "P3_DESC": "High volume",
     "P3_F1": "Unlimited calls", "P3_F2": "Dedicated support"}),
    ("storage", {"TITLE": "Cloud Storage", "DESC": "Cloud storage plan comparison",
     "SUBTITLE": "Store your files securely", "CLR": "#8b5cf6", "FEAT_BG": "#f5f3ff",
     "P1_NAME": "Personal", "P1_PRICE": "$3/mo", "P1_DESC": "For personal use",
     "P1_F1": "100GB space", "P1_F2": "File sharing",
     "P2_NAME": "Team", "P2_PRICE": "$12/mo", "P2_DESC": "Collaborate together",
     "P2_F1": "1TB space", "P2_F2": "Version history",
     "P3_NAME": "Business", "P3_PRICE": "$25/mo", "P3_DESC": "Enterprise grade",
     "P3_F1": "5TB space", "P3_F2": "Admin controls"}),
    ("email", {"TITLE": "Email Service", "DESC": "Email marketing plan pricing",
     "SUBTITLE": "Reach your audience", "CLR": "#ec4899", "FEAT_BG": "#fdf2f8",
     "P1_NAME": "Lite", "P1_PRICE": "$10/mo", "P1_DESC": "Small lists",
     "P1_F1": "500 subscribers", "P1_F2": "Basic templates",
     "P2_NAME": "Standard", "P2_PRICE": "$30/mo", "P2_DESC": "Growing lists",
     "P2_F1": "5K subscribers", "P2_F2": "A/B testing",
     "P3_NAME": "Premium", "P3_PRICE": "$75/mo", "P3_DESC": "Large lists",
     "P3_F1": "50K subscribers", "P3_F2": "Advanced analytics"}),
    ("vpn", {"TITLE": "VPN Plans", "DESC": "VPN subscription pricing table",
     "SUBTITLE": "Browse privately and securely", "CLR": "#0ea5e9", "FEAT_BG": "#f0f9ff",
     "P1_NAME": "Monthly", "P1_PRICE": "$12/mo", "P1_DESC": "Flexible plan",
     "P1_F1": "5 devices", "P1_F2": "50 servers",
     "P2_NAME": "Yearly", "P2_PRICE": "$5/mo", "P2_DESC": "Best value",
     "P2_F1": "10 devices", "P2_F2": "200 servers",
     "P3_NAME": "Lifetime", "P3_PRICE": "$99", "P3_DESC": "Pay once",
     "P3_F1": "Unlimited", "P3_F2": "All servers"}),
]:
    ex(f"gen-pricing-{n}.naze", cfg["DESC"], fill(PRICING_T, cfg))

# Four hand-crafted pricing examples (simpler format for variety)

ex("gen-pricing-simple.naze",
   "Simple two-tier pricing with free and paid plans",
   """-- Simple two-tier pricing
app "Simple Pricing" {
  state plan = "free"

  column padding: 24px, gap: 16px {
    heading "Pricing" color: #0f172a

    row gap: 16px {
      rect padding: 20px, color: #f8fafc, radius: 12px {
        column gap: 8px {
          text "Free" font-weight: bold, font-size: 20px
          text "$0" color: #16a34a, font-size: 32px
          text "3 projects" font-size: 14px
          text "Basic support" font-size: 14px
          rect width: 100px, height: 36px, color: #e2e8f0, radius: 6px {
            text "Current" color: #475569
            on click: set plan = "free"
          }
        }
      }
      rect padding: 20px, color: #eff6ff, radius: 12px {
        column gap: 8px {
          text "Pro" font-weight: bold, font-size: 20px
          text "$19/mo" color: #2563eb, font-size: 32px
          text "Unlimited projects" font-size: 14px
          text "Priority support" font-size: 14px
          rect width: 100px, height: 36px, color: #2563eb, radius: 6px {
            text "Upgrade" color: #ffffff
            on click: set plan = "pro"
          }
        }
      }
    }
  }
}""")

ex("gen-pricing-features.naze",
   "Feature comparison matrix for three plans",
   """-- Feature comparison matrix
app "Feature Matrix" {
  column padding: 24px, gap: 12px {
    heading "Compare Plans" color: #1e293b

    grid columns: 4, gap: 8px {
      text "Feature" font-weight: bold
      text "Basic" font-weight: bold, color: #64748b
      text "Pro" font-weight: bold, color: #2563eb
      text "Team" font-weight: bold, color: #7c3aed

      text "Users"
      text "1"
      text "5"
      text "Unlimited"

      text "Storage"
      text "1GB"
      text "50GB"
      text "500GB"

      text "API Access"
      text "No"
      text "Yes"
      text "Yes"

      text "Support"
      text "Email"
      text "Chat"
      text "Dedicated"

      text "Analytics"
      text "Basic"
      text "Advanced"
      text "Custom"
    }
  }
}""")

ex("gen-pricing-toggle.naze",
   "Pricing table with monthly/annual toggle switch",
   """-- Monthly vs annual pricing toggle
app "Toggle Pricing" {
  state annual = false
  state plan = "none"

  column padding: 24px, gap: 16px {
    heading "Pricing Plans" color: #0f172a

    row gap: 8px {
      text "Monthly" font-weight: bold
      rect width: 60px, height: 30px, color: #e2e8f0, radius: 15px {
        text "Toggle"
        on click: set annual = true
      }
      text "Annual" color: #64748b
    }

    if annual == false {
      row gap: 16px {
        rect padding: 16px, color: #f8fafc, radius: 8px {
          text "Basic: $9/mo" font-weight: bold
          on click: set plan = "basic-mo"
        }
        rect padding: 16px, color: #eff6ff, radius: 8px {
          text "Pro: $29/mo" font-weight: bold
          on click: set plan = "pro-mo"
        }
      }
    }

    if annual {
      row gap: 16px {
        rect padding: 16px, color: #f8fafc, radius: 8px {
          text "Basic: $90/yr" font-weight: bold
          on click: set plan = "basic-yr"
        }
        rect padding: 16px, color: #eff6ff, radius: 8px {
          text "Pro: $290/yr" font-weight: bold
          on click: set plan = "pro-yr"
        }
      }
    }

    if plan != "none" {
      text "Selected: {plan}" color: #16a34a
    }
  }
}""")

ex("gen-pricing-addon.naze",
   "Base plan with optional add-on features",
   """-- Pricing with add-on features
app "Plan Builder" {
  state base = 10
  state add-storage = false
  state add-support = false
  computed total = base + 5 + 10

  column padding: 24px, gap: 16px {
    heading "Build Your Plan" color: #6366f1
    text "Start at $10/mo and add what you need" color: #64748b

    rect padding: 16px, color: #eff6ff, radius: 8px {
      text "Base Plan - $10/mo" font-weight: bold
      text "5 projects, 5GB storage" color: #64748b, font-size: 14px
    }

    text "Add-ons:" font-weight: bold
    checkbox bind: add-storage, label: "Extra Storage (+$5/mo)"
    checkbox bind: add-support, label: "Priority Support (+$10/mo)"

    separator
    text "Estimated total: ${total}/mo" font-size: 20px, color: #6366f1
  }
}""")

# ─── 5. Contact Pages (10) ───────────────────────────────────────────────────

CONTACT_T = """-- __DESC__
app "__TITLE__" {
  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__
    text "__SUBTITLE__" color: #64748b

    grid columns: __COLS__, gap: 16px {
      rect padding: 16px, color: __BG__, radius: 8px {
        column gap: 4px {
          text "__N1__" font-weight: bold
          text "__R1__" color: #64748b, font-size: 14px
          text "__D1__" color: __CLR__, font-size: 14px
        }
      }
      rect padding: 16px, color: __BG__, radius: 8px {
        column gap: 4px {
          text "__N2__" font-weight: bold
          text "__R2__" color: #64748b, font-size: 14px
          text "__D2__" color: __CLR__, font-size: 14px
        }
      }
      rect padding: 16px, color: __BG__, radius: 8px {
        column gap: 4px {
          text "__N3__" font-weight: bold
          text "__R3__" color: #64748b, font-size: 14px
          text "__D3__" color: __CLR__, font-size: 14px
        }
      }
    }
  }
}"""

for n, cfg in [
    ("team", {"TITLE": "Our Team", "DESC": "Team directory with roles and contact info",
     "SUBTITLE": "Meet the team", "CLR": "#2563eb", "BG": "#f8fafc", "COLS": 3,
     "N1": "Alice Chen", "R1": "CEO", "D1": "alice@example.com",
     "N2": "Bob Singh", "R2": "CTO", "D2": "bob@example.com",
     "N3": "Carol Wu", "R3": "Design Lead", "D3": "carol@example.com"}),
    ("support", {"TITLE": "Contact Support", "DESC": "Support contact channels page",
     "SUBTITLE": "We are here to help", "CLR": "#16a34a", "BG": "#f0fdf4", "COLS": 3,
     "N1": "Email Support", "R1": "24h response time", "D1": "help@example.com",
     "N2": "Live Chat", "R2": "Available 9-5 EST", "D2": "chat.example.com",
     "N3": "Phone", "R3": "Mon-Fri 9am-5pm", "D3": "+1 555-0100"}),
    ("sales", {"TITLE": "Sales Team", "DESC": "Sales department contacts",
     "SUBTITLE": "Get in touch with sales", "CLR": "#f97316", "BG": "#fff7ed", "COLS": 3,
     "N1": "North America", "R1": "Enterprise Sales", "D1": "na@example.com",
     "N2": "Europe", "R2": "Regional Manager", "D2": "eu@example.com",
     "N3": "Asia Pacific", "R3": "Business Dev", "D3": "apac@example.com"}),
    ("office", {"TITLE": "Our Offices", "DESC": "Office locations directory",
     "SUBTITLE": "Visit us at our offices", "CLR": "#8b5cf6", "BG": "#f5f3ff", "COLS": 3,
     "N1": "San Francisco", "R1": "Headquarters", "D1": "123 Market St",
     "N2": "London", "R2": "EU Office", "D2": "45 Oxford St",
     "N3": "Tokyo", "R3": "Asia Office", "D3": "1-1 Shibuya"}),
    ("partners", {"TITLE": "Partners", "DESC": "Partner organizations directory",
     "SUBTITLE": "Our trusted partners", "CLR": "#0ea5e9", "BG": "#f0f9ff", "COLS": 3,
     "N1": "CloudCorp", "R1": "Infrastructure", "D1": "partner@cloudcorp.io",
     "N2": "DataFlow", "R2": "Analytics", "D2": "hello@dataflow.dev",
     "N3": "SecureNet", "R3": "Security", "D3": "info@securenet.com"}),
    ("advisors", {"TITLE": "Advisors", "DESC": "Advisory board listing",
     "SUBTITLE": "Our advisors and mentors", "CLR": "#d946ef", "BG": "#fdf2f8", "COLS": 3,
     "N1": "Dr. Jane Smith", "R1": "AI Research", "D1": "Stanford University",
     "N2": "Mark Johnson", "R2": "VC Partner", "D2": "Acme Ventures",
     "N3": "Lisa Park", "R3": "Product Strategy", "D3": "Ex-Google"}),
    ("hr", {"TITLE": "HR Contacts", "DESC": "Human resources department contacts",
     "SUBTITLE": "People operations team", "CLR": "#14b8a6", "BG": "#f0fdfa", "COLS": 3,
     "N1": "Recruiting", "R1": "Open positions", "D1": "jobs@example.com",
     "N2": "Benefits", "R2": "Insurance and perks", "D2": "benefits@example.com",
     "N3": "Culture", "R3": "Events and DEI", "D3": "culture@example.com"}),
]:
    ex(f"gen-contact-{n}.naze", cfg["DESC"], fill(CONTACT_T, cfg))

# Three hand-crafted contact examples

ex("gen-contact-card.naze",
   "Personal business card layout with name, title, and links",
   """-- Digital business card
app "Business Card" {
  column padding: 32px, gap: 12px {
    rect padding: 24px, color: #0f172a, radius: 16px {
      column gap: 8px {
        text "Alex Rivera" color: #ffffff, font-size: 28px, font-weight: bold
        text "Senior Engineer" color: #94a3b8, font-size: 16px
        separator
        text "alex@example.com" color: #60a5fa, font-size: 14px
        text "+1 555-0199" color: #94a3b8, font-size: 14px
        text "github.com/arivera" color: #94a3b8, font-size: 14px
      }
    }
  }
}""")

ex("gen-contact-form.naze",
   "Contact page combining form and company info side by side",
   """-- Contact page with form and info
app "Contact Us" {
  state sender = ""
  state subject = ""
  state body = ""
  state sent = false

  column padding: 24px, gap: 16px {
    heading "Get in Touch" color: #1e293b

    row gap: 24px {
      column gap: 12px {
        text "Send a Message" font-weight: bold
        input bind: sender, placeholder: "Your email"
        input bind: subject, placeholder: "Subject"
        input bind: body, placeholder: "Message"
        rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
          text "Send" color: #ffffff
          on click: set sent = true
        }
        if sent {
          text "Message sent!" color: #16a34a
        }
      }
      column gap: 8px {
        text "Company Info" font-weight: bold
        text "Acme Inc." color: #475569
        text "123 Main Street" color: #64748b
        text "hello@acme.com" color: #2563eb
        text "+1 800-555-0100" color: #64748b
      }
    }
  }
}""")

ex("gen-contact-social.naze",
   "Social media links page with platform icons",
   """-- Social links page
app "Social Links" {
  column padding: 24px, gap: 12px {
    heading "Follow Us" color: #0f172a
    text "Stay connected on social media" color: #64748b

    column gap: 8px {
      rect padding: 12px, color: #eff6ff, radius: 8px {
        link "Twitter - @ourcompany" href: "https://twitter.com/ourcompany"
      }
      rect padding: 12px, color: #f0fdf4, radius: 8px {
        link "GitHub - ourcompany" href: "https://github.com/ourcompany"
      }
      rect padding: 12px, color: #fef3c7, radius: 8px {
        link "LinkedIn - Our Company" href: "https://linkedin.com/company/ours"
      }
      rect padding: 12px, color: #fce7f3, radius: 8px {
        link "YouTube - OurChannel" href: "https://youtube.com/@ourchannel"
      }
    }
  }
}""")

# ─── 6. Activity Logs (10) ───────────────────────────────────────────────────

LOG_T = """-- __DESC__
app "__TITLE__" {
  state entries = [__ITEMS__]
  computed total-entries = entries | count

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__
    text "{total-entries} __LABEL__" color: #64748b

    each entry in entries {
      row padding: 10px, color: __BG__, radius: 6px, gap: 12px {
        text "{entry.time}" color: #94a3b8, font-size: 12px
        text "{entry.action}" font-size: 14px
        text "{entry.user}" color: __CLR__, font-size: 12px
      }
    }
  }
}"""

for n, cfg in [
    ("activity", {"TITLE": "Activity Log", "DESC": "User activity log with timestamps",
     "LABEL": "events logged", "CLR": "#2563eb", "BG": "#f8fafc",
     "ITEMS": '{time: "09:15", action: "Logged in", user: "Alice"}, {time: "09:32", action: "Created project", user: "Alice"}, {time: "10:05", action: "Invited Bob", user: "Alice"}, {time: "10:20", action: "Joined team", user: "Bob"}'}),
    ("audit", {"TITLE": "Audit Trail", "DESC": "Security audit trail for admin actions",
     "LABEL": "audit entries", "CLR": "#dc2626", "BG": "#fef2f2",
     "ITEMS": '{time: "08:00", action: "Password changed", user: "admin"}, {time: "08:15", action: "Role updated", user: "admin"}, {time: "09:00", action: "API key rotated", user: "admin"}, {time: "11:30", action: "User suspended", user: "admin"}'}),
    ("deploy", {"TITLE": "Deploy Log", "DESC": "Deployment history log",
     "LABEL": "deployments", "CLR": "#16a34a", "BG": "#f0fdf4",
     "ITEMS": '{time: "v2.1.0", action: "Production deploy", user: "CI"}, {time: "v2.0.9", action: "Hotfix applied", user: "Bob"}, {time: "v2.0.8", action: "Staging deploy", user: "CI"}, {time: "v2.0.7", action: "Rollback", user: "Alice"}'}),
    ("payment", {"TITLE": "Payment History", "DESC": "Payment transaction log",
     "LABEL": "transactions", "CLR": "#f59e0b", "BG": "#fefce8",
     "ITEMS": '{time: "Jan 1", action: "Subscription renewed", user: "$29.00"}, {time: "Dec 1", action: "Subscription renewed", user: "$29.00"}, {time: "Nov 15", action: "Plan upgraded", user: "$49.00"}, {time: "Nov 1", action: "Initial payment", user: "$29.00"}'}),
    ("error", {"TITLE": "Error Log", "DESC": "Application error log viewer",
     "LABEL": "errors recorded", "CLR": "#ef4444", "BG": "#fef2f2",
     "ITEMS": '{time: "14:22", action: "500 Internal Server Error", user: "/api/users"}, {time: "14:18", action: "404 Not Found", user: "/old-page"}, {time: "13:55", action: "429 Rate Limited", user: "/api/search"}, {time: "13:40", action: "503 Service Unavailable", user: "/api/data"}'}),
    ("commit", {"TITLE": "Commit Log", "DESC": "Git commit history viewer",
     "LABEL": "commits", "CLR": "#6366f1", "BG": "#eef2ff",
     "ITEMS": '{time: "2h ago", action: "Fix login bug", user: "alice"}, {time: "5h ago", action: "Add tests", user: "bob"}, {time: "1d ago", action: "Refactor auth", user: "carol"}, {time: "2d ago", action: "Update deps", user: "alice"}'}),
    ("notification", {"TITLE": "Notifications", "DESC": "Notification event history",
     "LABEL": "notifications", "CLR": "#0ea5e9", "BG": "#f0f9ff",
     "ITEMS": '{time: "Just now", action: "New comment on your post", user: "Social"}, {time: "1h ago", action: "Build succeeded", user: "CI/CD"}, {time: "3h ago", action: "Invoice ready", user: "Billing"}, {time: "1d ago", action: "Welcome aboard!", user: "System"}'}),
    ("access", {"TITLE": "Access Log", "DESC": "File access audit log",
     "LABEL": "access events", "CLR": "#8b5cf6", "BG": "#f5f3ff",
     "ITEMS": '{time: "10:30", action: "Viewed report.pdf", user: "Alice"}, {time: "10:15", action: "Downloaded data.csv", user: "Bob"}, {time: "09:45", action: "Edited config.yml", user: "Carol"}, {time: "09:30", action: "Shared notes.md", user: "Alice"}'}),
]:
    ex(f"gen-log-{n}.naze", cfg["DESC"], fill(LOG_T, cfg))

# Two hand-crafted log examples

ex("gen-log-realtime.naze",
   "Live event counter with auto-incrementing timer log",
   """-- Live event counter log
app "Live Events" {
  state event-count = 0
  state last-event = "none"

  timer event-tick: every 1s {
    set event-count = event-count + 1
  }

  column padding: 24px, gap: 16px {
    heading "Live Event Monitor" color: #0ea5e9
    text "Events captured: {event-count}" font-size: 24px, color: #0ea5e9

    match last-event {
      "none": text "Waiting for events..." color: #94a3b8
      _: text "Last: {last-event}" color: #475569
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: #ef4444, radius: 6px {
        text "Error" color: #ffffff
        on click: set last-event = "error"
      }
      rect width: 100px, height: 36px, color: #f59e0b, radius: 6px {
        text "Warning" color: #ffffff
        on click: set last-event = "warning"
      }
      rect width: 100px, height: 36px, color: #16a34a, radius: 6px {
        text "Info" color: #ffffff
        on click: set last-event = "info"
      }
    }
  }
}""")

ex("gen-log-changelog.naze",
   "Product changelog with version entries and dates",
   """-- Product changelog
app "Changelog" {
  state releases = [{ver: "2.3.0", date: "Feb 15", note: "Added dark mode"}, {ver: "2.2.1", date: "Feb 10", note: "Fixed login timeout"}, {ver: "2.2.0", date: "Jan 28", note: "New dashboard layout"}, {ver: "2.1.0", date: "Jan 15", note: "API v2 released"}]
  computed num-releases = releases | count

  column padding: 24px, gap: 16px {
    heading "Changelog" color: #1e293b
    text "{num-releases} releases" color: #64748b

    each rel in releases {
      rect padding: 14px, color: #f8fafc, radius: 8px {
        row gap: 12px {
          text "v{rel.ver}" font-weight: bold, color: #2563eb
          text "{rel.date}" color: #94a3b8, font-size: 12px
        }
        text "{rel.note}" color: #475569, font-size: 14px
      }
    }
  }
}""")

# ─── 7. KPI / Metric Dashboards (10) ─────────────────────────────────────────

METRIC_T = """-- __DESC__
app "__TITLE__" {
  state __M1__ = __V1__
  state __M2__ = __V2__
  state __M3__ = __V3__
  state __M4__ = __V4__

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: #0f172a

    grid columns: 2, gap: 12px {
      rect padding: 16px, color: __BG1__, radius: 10px {
        text "__L1__" color: #64748b, font-size: 13px
        text "{__M1__}" font-size: 28px, color: __C1__
      }
      rect padding: 16px, color: __BG2__, radius: 10px {
        text "__L2__" color: #64748b, font-size: 13px
        text "{__M2__}" font-size: 28px, color: __C2__
      }
      rect padding: 16px, color: __BG3__, radius: 10px {
        text "__L3__" color: #64748b, font-size: 13px
        text "{__M3__}" font-size: 28px, color: __C3__
      }
      rect padding: 16px, color: __BG4__, radius: 10px {
        text "__L4__" color: #64748b, font-size: 13px
        text "{__M4__}" font-size: 28px, color: __C4__
      }
    }
  }
}"""

for n, cfg in [
    ("sales", {"TITLE": "Sales Dashboard", "DESC": "Sales KPI dashboard with four metrics",
     "M1": "deals-closed", "V1": 47, "L1": "Deals Closed", "BG1": "#eff6ff", "C1": "#1e40af",
     "M2": "pipeline", "V2": 128, "L2": "In Pipeline", "BG2": "#f0fdf4", "C2": "#166534",
     "M3": "avg-deal", "V3": 8500, "L3": "Avg Deal Size", "BG3": "#fef3c7", "C3": "#92400e",
     "M4": "win-rate", "V4": 36, "L4": "Win Rate %", "BG4": "#fce7f3", "C4": "#9d174d"}),
    ("web", {"TITLE": "Web Analytics", "DESC": "Website analytics summary dashboard",
     "M1": "pageviews", "V1": 24500, "L1": "Page Views", "BG1": "#f0f9ff", "C1": "#0369a1",
     "M2": "sessions", "V2": 8200, "L2": "Sessions", "BG2": "#ecfdf5", "C2": "#047857",
     "M3": "bounce", "V3": 42, "L3": "Bounce Rate %", "BG3": "#fff7ed", "C3": "#c2410c",
     "M4": "duration", "V4": 185, "L4": "Avg Duration (s)", "BG4": "#fdf2f8", "C4": "#be185d"}),
    ("devops", {"TITLE": "DevOps Metrics", "DESC": "Infrastructure health dashboard",
     "M1": "uptime", "V1": 99, "L1": "Uptime %", "BG1": "#f0fdf4", "C1": "#15803d",
     "M2": "latency", "V2": 45, "L2": "Latency (ms)", "BG2": "#eff6ff", "C2": "#1d4ed8",
     "M3": "errors-today", "V3": 3, "L3": "Errors Today", "BG3": "#fef2f2", "C3": "#dc2626",
     "M4": "deploys", "V4": 12, "L4": "Deploys / Week", "BG4": "#f5f3ff", "C4": "#7c3aed"}),
    ("hr", {"TITLE": "HR Dashboard", "DESC": "Human resources KPI overview",
     "M1": "headcount", "V1": 156, "L1": "Headcount", "BG1": "#eff6ff", "C1": "#2563eb",
     "M2": "open-roles", "V2": 14, "L2": "Open Roles", "BG2": "#fef3c7", "C2": "#d97706",
     "M3": "retention", "V3": 92, "L3": "Retention %", "BG3": "#f0fdf4", "C3": "#16a34a",
     "M4": "satisfaction", "V4": 87, "L4": "Satisfaction %", "BG4": "#fdf2f8", "C4": "#db2777"}),
    ("finance", {"TITLE": "Finance Overview", "DESC": "Financial metrics summary",
     "M1": "revenue", "V1": 245000, "L1": "Revenue ($)", "BG1": "#f0fdf4", "C1": "#166534",
     "M2": "expenses", "V2": 178000, "L2": "Expenses ($)", "BG2": "#fef2f2", "C2": "#dc2626",
     "M3": "profit-margin", "V3": 27, "L3": "Profit Margin %", "BG3": "#eff6ff", "C3": "#1e40af",
     "M4": "runway", "V4": 18, "L4": "Runway (months)", "BG4": "#fef3c7", "C4": "#92400e"}),
    ("support", {"TITLE": "Support Metrics", "DESC": "Customer support performance dashboard",
     "M1": "open-tickets", "V1": 23, "L1": "Open Tickets", "BG1": "#fef3c7", "C1": "#d97706",
     "M2": "resolved-today", "V2": 18, "L2": "Resolved Today", "BG2": "#f0fdf4", "C2": "#16a34a",
     "M3": "avg-response", "V3": 4, "L3": "Avg Response (h)", "BG3": "#eff6ff", "C3": "#2563eb",
     "M4": "csat", "V4": 94, "L4": "CSAT Score %", "BG4": "#fce7f3", "C4": "#db2777"}),
    ("marketing", {"TITLE": "Marketing Metrics", "DESC": "Marketing campaign performance dashboard",
     "M1": "leads", "V1": 340, "L1": "New Leads", "BG1": "#eff6ff", "C1": "#1e40af",
     "M2": "conversions", "V2": 52, "L2": "Conversions", "BG2": "#f0fdf4", "C2": "#15803d",
     "M3": "ctr", "V3": 3, "L3": "CTR %", "BG3": "#fff7ed", "C3": "#ea580c",
     "M4": "spend", "V4": 12500, "L4": "Ad Spend ($)", "BG4": "#fef2f2", "C4": "#dc2626"}),
    ("product", {"TITLE": "Product Metrics", "DESC": "Product usage analytics dashboard",
     "M1": "dau", "V1": 5200, "L1": "Daily Active Users", "BG1": "#f0f9ff", "C1": "#0284c7",
     "M2": "mau", "V2": 28000, "L2": "Monthly Active", "BG2": "#ecfdf5", "C2": "#059669",
     "M3": "nps", "V3": 72, "L3": "NPS Score", "BG3": "#f5f3ff", "C3": "#7c3aed",
     "M4": "churn", "V4": 2, "L4": "Churn Rate %", "BG4": "#fef2f2", "C4": "#ef4444"}),
]:
    ex(f"gen-metric-{n}.naze", cfg["DESC"], fill(METRIC_T, cfg))

# Two hand-crafted metric examples

ex("gen-metric-live.naze",
   "Live metrics dashboard with auto-updating counter",
   """-- Live updating metrics
app "Live Metrics" {
  state requests = 0
  state active-users = 42
  computed rps = requests

  timer req-tick: every 1s {
    set requests = requests + 3
  }

  column padding: 24px, gap: 16px {
    heading "Live System Metrics" color: #0ea5e9

    grid columns: 3, gap: 12px {
      rect padding: 16px, color: #f0f9ff, radius: 10px {
        text "Requests" color: #64748b, font-size: 13px
        text "{requests}" font-size: 32px, color: #0ea5e9
      }
      rect padding: 16px, color: #f0fdf4, radius: 10px {
        text "Active Users" color: #64748b, font-size: 13px
        text "{active-users}" font-size: 32px, color: #16a34a
      }
      rect padding: 16px, color: #f5f3ff, radius: 10px {
        text "Req/sec" color: #64748b, font-size: 13px
        text "{rps}" font-size: 32px, color: #7c3aed
      }
    }
  }
}""")

ex("gen-metric-comparison.naze",
   "This week vs last week metric comparison cards",
   """-- Week over week comparison
app "Weekly Comparison" {
  state this-week = 1250
  state last-week = 980
  computed growth = this-week - last-week

  column padding: 24px, gap: 16px {
    heading "Performance Comparison" color: #1e293b

    row gap: 16px {
      rect padding: 20px, color: #f0fdf4, radius: 10px {
        column gap: 4px {
          text "This Week" color: #64748b, font-size: 13px
          text "{this-week}" font-size: 32px, color: #16a34a, font-weight: bold
        }
      }
      rect padding: 20px, color: #f8fafc, radius: 10px {
        column gap: 4px {
          text "Last Week" color: #64748b, font-size: 13px
          text "{last-week}" font-size: 32px, color: #475569
        }
      }
      rect padding: 20px, color: #eff6ff, radius: 10px {
        column gap: 4px {
          text "Growth" color: #64748b, font-size: 13px
          text "+{growth}" font-size: 32px, color: #2563eb, font-weight: bold
        }
      }
    }
  }
}""")

# ─── 8. Product Catalogs (10) ────────────────────────────────────────────────

CATALOG_T = """-- __DESC__
app "__TITLE__" {
  state items = [__ITEMS__]
  computed item-count = items | count
  state view = "grid"

  column padding: 24px, gap: 16px {
    heading "__TITLE__" color: __CLR__
    text "{item-count} __LABEL__" color: #64748b

    grid columns: __COLS__, gap: 12px {
      each item in items | sort-by __SORT__ {
        rect padding: 14px, color: __BG__, radius: 10px {
          column gap: 6px {
            text "{item.name}" font-weight: bold, font-size: 16px
            text "{item.__F2__}" color: __CLR__, font-size: 14px
            text "{item.__F3__}" color: #94a3b8, font-size: 12px
          }
        }
      }
    }
  }
}"""

for n, cfg in [
    ("electronics", {"TITLE": "Electronics", "DESC": "Electronics product catalog",
     "LABEL": "products", "CLR": "#2563eb", "BG": "#f8fafc", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "brand",
     "ITEMS": '{name: "Laptop Pro", price: "$1299", brand: "TechCo"}, {name: "Wireless Earbuds", price: "$79", brand: "AudioMax"}, {name: "Smart Watch", price: "$349", brand: "TechCo"}, {name: "USB-C Hub", price: "$49", brand: "ConnectPlus"}'}),
    ("books", {"TITLE": "Book Catalog", "DESC": "Bookstore catalog with author and price",
     "LABEL": "books available", "CLR": "#92400e", "BG": "#fef3c7", "COLS": 2, "SORT": "name",
     "F2": "author", "F3": "price",
     "ITEMS": '{name: "Clean Code", author: "R. Martin", price: "$35"}, {name: "The Pragmatic Programmer", author: "Hunt & Thomas", price: "$42"}, {name: "Refactoring", author: "M. Fowler", price: "$38"}, {name: "DDIA", author: "M. Kleppmann", price: "$45"}'}),
    ("clothing", {"TITLE": "Clothing Store", "DESC": "Fashion items catalog",
     "LABEL": "items in stock", "CLR": "#db2777", "BG": "#fdf2f8", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "category",
     "ITEMS": '{name: "Cotton Tee", price: "$25", category: "Tops"}, {name: "Slim Jeans", price: "$65", category: "Bottoms"}, {name: "Wool Sweater", price: "$85", category: "Tops"}, {name: "Canvas Sneakers", price: "$55", category: "Shoes"}'}),
    ("food", {"TITLE": "Menu", "DESC": "Restaurant menu catalog",
     "LABEL": "dishes", "CLR": "#ea580c", "BG": "#fff7ed", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "category",
     "ITEMS": '{name: "Margherita Pizza", price: "$14", category: "Mains"}, {name: "Caesar Salad", price: "$10", category: "Starters"}, {name: "Tiramisu", price: "$8", category: "Desserts"}, {name: "Bruschetta", price: "$9", category: "Starters"}'}),
    ("software", {"TITLE": "App Store", "DESC": "Software applications catalog",
     "LABEL": "apps listed", "CLR": "#7c3aed", "BG": "#f5f3ff", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "category",
     "ITEMS": '{name: "Photo Editor", price: "$9.99", category: "Creative"}, {name: "Task Manager", price: "Free", category: "Productivity"}, {name: "Code IDE", price: "$29", category: "Developer"}, {name: "Music Player", price: "$4.99", category: "Media"}'}),
    ("furniture", {"TITLE": "Furniture Shop", "DESC": "Home furniture catalog",
     "LABEL": "pieces", "CLR": "#78716c", "BG": "#fafaf9", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "material",
     "ITEMS": '{name: "Oak Desk", price: "$450", material: "Wood"}, {name: "Ergonomic Chair", price: "$380", material: "Mesh"}, {name: "Bookshelf", price: "$220", material: "Pine"}, {name: "Side Table", price: "$120", material: "Bamboo"}'}),
    ("plants", {"TITLE": "Plant Nursery", "DESC": "Indoor plant catalog",
     "LABEL": "plants available", "CLR": "#15803d", "BG": "#f0fdf4", "COLS": 2, "SORT": "name",
     "F2": "price", "F3": "care",
     "ITEMS": '{name: "Monstera", price: "$35", care: "Medium light"}, {name: "Snake Plant", price: "$20", care: "Low light"}, {name: "Pothos", price: "$15", care: "Any light"}, {name: "Fiddle Leaf", price: "$45", care: "Bright light"}'}),
]:
    ex(f"gen-catalog-{n}.naze", cfg["DESC"], fill(CATALOG_T, cfg))

# Three hand-crafted catalog examples

ex("gen-catalog-filtered.naze",
   "Product catalog with category filter buttons",
   """-- Filtered product catalog
app "Filtered Catalog" {
  state category = "all"
  state products = [{name: "Widget A", cat: "tools", price: "$12"}, {name: "Gadget B", cat: "tech", price: "$29"}, {name: "Tool C", cat: "tools", price: "$18"}, {name: "Device D", cat: "tech", price: "$45"}]

  column padding: 24px, gap: 16px {
    heading "Products" color: #1e293b

    row gap: 8px {
      rect width: 80px, height: 32px, color: #2563eb, radius: 6px {
        text "All" color: #ffffff
        on click: set category = "all"
      }
      rect width: 80px, height: 32px, color: #e2e8f0, radius: 6px {
        text "Tools"
        on click: set category = "tools"
      }
      rect width: 80px, height: 32px, color: #e2e8f0, radius: 6px {
        text "Tech"
        on click: set category = "tech"
      }
    }

    text "Showing: {category}" color: #64748b

    each prod in products {
      rect padding: 12px, color: #f8fafc, radius: 8px {
        row gap: 12px {
          text "{prod.name}" font-weight: bold
          text "{prod.price}" color: #2563eb
          text "{prod.cat}" color: #94a3b8, font-size: 12px
        }
      }
    }
  }
}""")

ex("gen-catalog-search.naze",
   "Searchable catalog with input filter and item count",
   """-- Searchable product catalog
app "Search Catalog" {
  state search = ""
  state products = [{name: "Laptop", sku: "LP-001"}, {name: "Monitor", sku: "MN-002"}, {name: "Keyboard", sku: "KB-003"}, {name: "Mouse", sku: "MS-004"}, {name: "Webcam", sku: "WC-005"}]
  computed total = products | count

  column padding: 24px, gap: 16px {
    heading "Product Search" color: #0f172a
    input bind: search, placeholder: "Search products..."
    text "{total} products in catalog" color: #64748b

    each prod in products | sort-by name {
      row padding: 10px, color: #f1f5f9, radius: 6px, gap: 12px {
        text "{prod.name}" font-weight: bold
        text "{prod.sku}" color: #94a3b8, font-size: 12px
      }
    }
  }
}""")

ex("gen-catalog-detail.naze",
   "Multi-page catalog with product detail page",
   """-- Catalog with detail page
app "Shop" {
  state items = [{name: "Alpha Widget", desc: "Premium quality"}, {name: "Beta Gadget", desc: "Best value"}, {name: "Gamma Tool", desc: "Professional grade"}]

  column padding: 24px, gap: 16px {
    heading "Our Products" color: #1e293b
    text "Browse our collection" color: #64748b

    each item in items {
      rect padding: 16px, color: #f8fafc, radius: 10px {
        text "{item.name}" font-weight: bold, font-size: 18px
        text "{item.desc}" color: #64748b
        link "View details", to: "/product"
      }
    }
  }
}

page "/product" {
  column padding: 24px, gap: 16px {
    heading "Product Details" color: #1e293b
    text "Detailed product information goes here" color: #475569
    link "Back to catalog", to: "/"
  }
}""")

# ─── 9. Hero Banners & CTA Sections (10) ─────────────────────────────────────

BANNER_T = """-- __DESC__
app "__TITLE__" {
  column {
    rect width: 800px, height: __HEIGHT__px, color: __BG__, radius: __RAD__px {
      column padding: __PAD__px, gap: 12px {
        text "__HEADLINE__" color: __FG__, font-size: __HSIZE__px, font-weight: bold
        text "__SUBTEXT__" color: __SUB_CLR__, font-size: 16px
        rect width: __BTN_W__px, height: 44px, color: __BTN_CLR__, radius: 8px {
          text "__BTN__" color: __BTN_FG__
        }
      }
    }
  }
}"""

for n, cfg in [
    ("startup", {"TITLE": "Startup Landing", "DESC": "Startup hero banner with CTA button",
     "HEIGHT": 320, "BG": "#0f172a", "RAD": 0, "PAD": 48,
     "HEADLINE": "Build the future, today", "FG": "#ffffff", "HSIZE": 36,
     "SUBTEXT": "The platform for modern teams to ship faster.",
     "SUB_CLR": "#94a3b8", "BTN": "Get Started", "BTN_W": 140,
     "BTN_CLR": "#3b82f6", "BTN_FG": "#ffffff"}),
    ("saas", {"TITLE": "SaaS Hero", "DESC": "SaaS product hero section",
     "HEIGHT": 280, "BG": "#1e40af", "RAD": 0, "PAD": 40,
     "HEADLINE": "Automate your workflow", "FG": "#ffffff", "HSIZE": 32,
     "SUBTEXT": "Save hours every week with smart automation.",
     "SUB_CLR": "#bfdbfe", "BTN": "Start Free Trial", "BTN_W": 160,
     "BTN_CLR": "#ffffff", "BTN_FG": "#1e40af"}),
    ("ecommerce", {"TITLE": "Shop Banner", "DESC": "E-commerce promotional banner",
     "HEIGHT": 260, "BG": "#fef3c7", "RAD": 16, "PAD": 40,
     "HEADLINE": "Summer Sale - 30% Off", "FG": "#92400e", "HSIZE": 34,
     "SUBTEXT": "Limited time offer on all collections.",
     "SUB_CLR": "#a16207", "BTN": "Shop Now", "BTN_W": 120,
     "BTN_CLR": "#92400e", "BTN_FG": "#ffffff"}),
    ("launch", {"TITLE": "Product Launch", "DESC": "New product launch announcement banner",
     "HEIGHT": 300, "BG": "#6366f1", "RAD": 0, "PAD": 48,
     "HEADLINE": "Introducing v3.0", "FG": "#ffffff", "HSIZE": 38,
     "SUBTEXT": "Completely redesigned from the ground up.",
     "SUB_CLR": "#c7d2fe", "BTN": "Explore Features", "BTN_W": 170,
     "BTN_CLR": "#ffffff", "BTN_FG": "#4f46e5"}),
    ("event", {"TITLE": "Event Banner", "DESC": "Conference event announcement banner",
     "HEIGHT": 280, "BG": "#0f172a", "RAD": 12, "PAD": 40,
     "HEADLINE": "DevConf 2025", "FG": "#f0abfc", "HSIZE": 36,
     "SUBTEXT": "Join 5000+ developers. March 15-17.",
     "SUB_CLR": "#d8b4fe", "BTN": "Register Now", "BTN_W": 150,
     "BTN_CLR": "#a855f7", "BTN_FG": "#ffffff"}),
    ("newsletter", {"TITLE": "Newsletter CTA", "DESC": "Newsletter signup call-to-action banner",
     "HEIGHT": 200, "BG": "#ecfdf5", "RAD": 12, "PAD": 32,
     "HEADLINE": "Stay in the loop", "FG": "#065f46", "HSIZE": 28,
     "SUBTEXT": "Weekly tips and insights delivered to your inbox.",
     "SUB_CLR": "#047857", "BTN": "Subscribe", "BTN_W": 120,
     "BTN_CLR": "#059669", "BTN_FG": "#ffffff"}),
    ("hiring", {"TITLE": "Hiring Banner", "DESC": "We are hiring announcement banner",
     "HEIGHT": 220, "BG": "#eff6ff", "RAD": 12, "PAD": 36,
     "HEADLINE": "Join Our Team", "FG": "#1e3a8a", "HSIZE": 30,
     "SUBTEXT": "We are looking for passionate builders.",
     "SUB_CLR": "#1d4ed8", "BTN": "View Openings", "BTN_W": 150,
     "BTN_CLR": "#2563eb", "BTN_FG": "#ffffff"}),
    ("maintenance", {"TITLE": "Maintenance Notice", "DESC": "Scheduled maintenance announcement",
     "HEIGHT": 180, "BG": "#fef2f2", "RAD": 8, "PAD": 28,
     "HEADLINE": "Scheduled Maintenance", "FG": "#991b1b", "HSIZE": 26,
     "SUBTEXT": "We will be down for upgrades on Sunday 2am-4am EST.",
     "SUB_CLR": "#dc2626", "BTN": "Learn More", "BTN_W": 120,
     "BTN_CLR": "#ef4444", "BTN_FG": "#ffffff"}),
]:
    ex(f"gen-banner-{n}.naze", cfg["DESC"], fill(BANNER_T, cfg))

# Two hand-crafted banner examples

ex("gen-banner-countdown.naze",
   "Sale countdown banner with live timer",
   """-- Sale countdown banner
app "Flash Sale" {
  state hours-left = 23
  state minutes-left = 59

  timer sale-tick: every 1s {
    set minutes-left = minutes-left - 1
  }

  column {
    rect width: 800px, height: 240px, color: #dc2626, radius: 0px {
      column padding: 40px, gap: 12px {
        text "FLASH SALE" color: #ffffff, font-size: 40px, font-weight: bold
        text "Ends in {hours-left}h {minutes-left}m" color: #fecaca, font-size: 20px
        text "Up to 50% off everything" color: #fca5a5, font-size: 16px
        rect width: 140px, height: 44px, color: #ffffff, radius: 8px {
          text "Shop Sale" color: #dc2626, font-weight: bold
        }
      }
    }
  }
}""")

ex("gen-banner-testimonial.naze",
   "Customer testimonial banner with quote",
   """-- Testimonial banner
app "Testimonial" {
  column {
    rect width: 800px, height: 260px, color: #f8fafc {
      column padding: 40px, gap: 16px {
        text "What our customers say" color: #64748b, font-size: 14px
        text "This product changed how our team works. We shipped 3x faster in the first month." color: #0f172a, font-size: 20px
        row gap: 8px {
          text "- Sarah Chen" font-weight: bold, color: #1e293b
          text "VP of Engineering, Acme Corp" color: #64748b, font-size: 14px
        }
      }
    }
  }
}""")

# ─── 10. Footer Layouts (10) ─────────────────────────────────────────────────

FOOTER_T = """-- __DESC__
app "__TITLE__" {
  column {
    spacer height: 40px

    rect width: 800px, color: __BG__ {
      column padding: 32px, gap: 16px {
        row gap: __GAP__px {
          column gap: 6px {
            text "__C1_TITLE__" font-weight: bold, color: __FG__
            text "__C1_L1__" color: __LINK__, font-size: 14px
            text "__C1_L2__" color: __LINK__, font-size: 14px
            text "__C1_L3__" color: __LINK__, font-size: 14px
          }
          column gap: 6px {
            text "__C2_TITLE__" font-weight: bold, color: __FG__
            text "__C2_L1__" color: __LINK__, font-size: 14px
            text "__C2_L2__" color: __LINK__, font-size: 14px
            text "__C2_L3__" color: __LINK__, font-size: 14px
          }
          column gap: 6px {
            text "__C3_TITLE__" font-weight: bold, color: __FG__
            text "__C3_L1__" color: __LINK__, font-size: 14px
            text "__C3_L2__" color: __LINK__, font-size: 14px
            text "__C3_L3__" color: __LINK__, font-size: 14px
          }
        }
        separator
        text "__COPYRIGHT__" color: __LINK__, font-size: 12px
      }
    }
  }
}"""

for n, cfg in [
    ("company", {"TITLE": "Company Footer", "DESC": "Company website footer with sitemap columns",
     "BG": "#0f172a", "FG": "#ffffff", "LINK": "#94a3b8", "GAP": 48,
     "C1_TITLE": "Product", "C1_L1": "Features", "C1_L2": "Pricing", "C1_L3": "Changelog",
     "C2_TITLE": "Company", "C2_L1": "About Us", "C2_L2": "Careers", "C2_L3": "Blog",
     "C3_TITLE": "Support", "C3_L1": "Help Center", "C3_L2": "Contact", "C3_L3": "Status",
     "COPYRIGHT": "2025 Acme Inc. All rights reserved."}),
    ("minimal", {"TITLE": "Minimal Footer", "DESC": "Clean minimal website footer",
     "BG": "#f8fafc", "FG": "#0f172a", "LINK": "#64748b", "GAP": 48,
     "C1_TITLE": "Links", "C1_L1": "Home", "C1_L2": "About", "C1_L3": "Contact",
     "C2_TITLE": "Legal", "C2_L1": "Privacy", "C2_L2": "Terms", "C2_L3": "Cookies",
     "C3_TITLE": "Social", "C3_L1": "Twitter", "C3_L2": "GitHub", "C3_L3": "LinkedIn",
     "COPYRIGHT": "Made with Naze"}),
    ("saas", {"TITLE": "SaaS Footer", "DESC": "SaaS application footer with resource links",
     "BG": "#1e293b", "FG": "#e2e8f0", "LINK": "#94a3b8", "GAP": 40,
     "C1_TITLE": "Product", "C1_L1": "Dashboard", "C1_L2": "API Docs", "C1_L3": "Integrations",
     "C2_TITLE": "Resources", "C2_L1": "Documentation", "C2_L2": "Tutorials", "C2_L3": "Community",
     "C3_TITLE": "Company", "C3_L1": "About", "C3_L2": "Press Kit", "C3_L3": "Investors",
     "COPYRIGHT": "2025 SaaSCo. Built with love."}),
    ("ecommerce", {"TITLE": "Shop Footer", "DESC": "E-commerce store footer",
     "BG": "#fafaf9", "FG": "#1c1917", "LINK": "#78716c", "GAP": 48,
     "C1_TITLE": "Shop", "C1_L1": "New Arrivals", "C1_L2": "Best Sellers", "C1_L3": "Sale",
     "C2_TITLE": "Help", "C2_L1": "Shipping", "C2_L2": "Returns", "C2_L3": "Size Guide",
     "C3_TITLE": "About", "C3_L1": "Our Story", "C3_L2": "Sustainability", "C3_L3": "Stores",
     "COPYRIGHT": "2025 ShopCo. Free shipping on orders over $50."}),
    ("developer", {"TITLE": "Dev Footer", "DESC": "Developer tools footer with documentation links",
     "BG": "#0f172a", "FG": "#e2e8f0", "LINK": "#60a5fa", "GAP": 40,
     "C1_TITLE": "Docs", "C1_L1": "Quick Start", "C1_L2": "API Reference", "C1_L3": "Examples",
     "C2_TITLE": "Community", "C2_L1": "Discord", "C2_L2": "Forum", "C2_L3": "Stack Overflow",
     "C3_TITLE": "Open Source", "C3_L1": "GitHub", "C3_L2": "Contributing", "C3_L3": "License",
     "COPYRIGHT": "Open source under MIT License."}),
    ("blog", {"TITLE": "Blog Footer", "DESC": "Blog website footer with category links",
     "BG": "#1e1b4b", "FG": "#e0e7ff", "LINK": "#a5b4fc", "GAP": 48,
     "C1_TITLE": "Categories", "C1_L1": "Technology", "C1_L2": "Design", "C1_L3": "Business",
     "C2_TITLE": "Popular", "C2_L1": "Getting Started", "C2_L2": "Best Practices", "C2_L3": "Case Studies",
     "C3_TITLE": "More", "C3_L1": "Newsletter", "C3_L2": "RSS Feed", "C3_L3": "Sponsor",
     "COPYRIGHT": "2025 The Dev Blog. Written by developers."}),
    ("agency", {"TITLE": "Agency Footer", "DESC": "Creative agency footer",
     "BG": "#18181b", "FG": "#fafafa", "LINK": "#a1a1aa", "GAP": 40,
     "C1_TITLE": "Services", "C1_L1": "Web Design", "C1_L2": "Branding", "C1_L3": "Marketing",
     "C2_TITLE": "Work", "C2_L1": "Portfolio", "C2_L2": "Case Studies", "C2_L3": "Testimonials",
     "C3_TITLE": "Connect", "C3_L1": "Email Us", "C3_L2": "Twitter", "C3_L3": "Dribbble",
     "COPYRIGHT": "2025 Studio Creative. All rights reserved."}),
]:
    ex(f"gen-footer-{n}.naze", cfg["DESC"], fill(FOOTER_T, cfg))

# Three hand-crafted footer examples

ex("gen-footer-simple.naze",
   "Simple one-line footer with copyright text",
   """-- Simple single-line footer
app "Simple Footer" {
  column {
    spacer height: 100px
    heading "My Website" font-size: 24px
    text "Main content goes here" color: #64748b
    spacer height: 100px

    rect width: 800px, height: 60px, color: #f1f5f9 {
      text "2025 MyCompany. All rights reserved." color: #64748b, font-size: 13px
    }
  }
}""")

ex("gen-footer-links.naze",
   "Footer with horizontal link row and separator",
   """-- Footer with horizontal links
app "Link Footer" {
  column gap: 16px {
    heading "Page Title"
    text "Page content above the footer" color: #64748b

    spacer height: 60px
    separator

    row gap: 24px {
      link "Home" href: "/"
      link "About" href: "/about"
      link "Blog" href: "/blog"
      link "Contact" href: "/contact"
      link "Privacy" href: "/privacy"
    }

    text "Built with Naze" color: #94a3b8, font-size: 12px
  }
}""")

ex("gen-footer-newsletter.naze",
   "Footer with newsletter signup form and social links",
   """-- Footer with newsletter signup
app "Newsletter Footer" {
  state email = ""
  state subscribed = false

  column {
    heading "Welcome"
    text "Site content here" color: #64748b
    spacer height: 80px

    rect width: 800px, color: #1e293b {
      column padding: 32px, gap: 16px {
        row gap: 48px {
          column gap: 8px {
            text "Newsletter" color: #ffffff, font-weight: bold
            text "Get updates in your inbox" color: #94a3b8, font-size: 14px
            row gap: 8px {
              input bind: email, placeholder: "Enter email"
              rect width: 100px, height: 36px, color: #3b82f6, radius: 6px {
                text "Join" color: #ffffff
                on click: set subscribed = true
              }
            }
            if subscribed {
              text "Subscribed!" color: #4ade80, font-size: 14px
            }
          }
          column gap: 6px {
            text "Follow Us" color: #ffffff, font-weight: bold
            text "Twitter" color: #94a3b8, font-size: 14px
            text "GitHub" color: #94a3b8, font-size: 14px
            text "Discord" color: #94a3b8, font-size: 14px
          }
        }
        separator
        text "2025 Company Inc." color: #64748b, font-size: 12px
      }
    }
  }
}""")


# ═══════════════════════════════════════════════════════════════════════════════
# Batch C: Sidebar, Modal, Tab, Accordion, Toast, Avatar, Tag, Empty, Hero, Toolbar
# (100 examples)
# ═══════════════════════════════════════════════════════════════════════════════


# ─── 1. Sidebar navigation patterns (10) ─────────────────────────────────────

SIDEBAR_T = """-- __DESC__
app "__TITLE__" {
  state active-page = "__DEFAULT__"

  row gap: 0px {
    rect width: 220px, height: 500px, color: __SIDEBAR_BG__ {
      column padding: 16px, gap: 4px {
        text "__BRAND__" font-weight: bold, font-size: 18px, color: __BRAND_CLR__
        spacer height: 16px
        rect padding: 10px, radius: 6px {
          text "__NAV1__" color: __TEXT_CLR__
          on click: set active-page = "__K1__"
        }
        rect padding: 10px, radius: 6px {
          text "__NAV2__" color: __TEXT_CLR__
          on click: set active-page = "__K2__"
        }
        rect padding: 10px, radius: 6px {
          text "__NAV3__" color: __TEXT_CLR__
          on click: set active-page = "__K3__"
        }
        rect padding: 10px, radius: 6px {
          text "__NAV4__" color: __TEXT_CLR__
          on click: set active-page = "__K4__"
        }
      }
    }

    column padding: 24px, gap: 16px {
      match active-page {
        "__K1__": heading "__NAV1__"
        "__K2__": heading "__NAV2__"
        "__K3__": heading "__NAV3__"
        "__K4__": heading "__NAV4__"
        _: heading "Select a page"
      }
      text "Active: {active-page}" color: #64748b
    }
  }
}"""

for n, cfg in [
    ("admin", {"TITLE": "Admin Panel", "DESC": "Admin sidebar with section navigation", "DEFAULT": "dashboard", "BRAND": "Admin", "BRAND_CLR": "#ffffff", "SIDEBAR_BG": "#1e293b", "TEXT_CLR": "#cbd5e1", "NAV1": "Dashboard", "K1": "dashboard", "NAV2": "Users", "K2": "users", "NAV3": "Settings", "K3": "settings", "NAV4": "Reports", "K4": "reports"}),
    ("docs", {"TITLE": "Documentation", "DESC": "Documentation sidebar with topic navigation", "DEFAULT": "intro", "BRAND": "Docs", "BRAND_CLR": "#2563eb", "SIDEBAR_BG": "#f8fafc", "TEXT_CLR": "#334155", "NAV1": "Introduction", "K1": "intro", "NAV2": "Quick Start", "K2": "quickstart", "NAV3": "API Reference", "K3": "api", "NAV4": "Examples", "K4": "examples"}),
    ("email", {"TITLE": "Email Client", "DESC": "Email client sidebar with folder navigation", "DEFAULT": "inbox", "BRAND": "Mail", "BRAND_CLR": "#dc2626", "SIDEBAR_BG": "#fafafa", "TEXT_CLR": "#374151", "NAV1": "Inbox", "K1": "inbox", "NAV2": "Sent", "K2": "sent", "NAV3": "Drafts", "K3": "drafts", "NAV4": "Trash", "K4": "trash"}),
    ("music", {"TITLE": "Music Player", "DESC": "Music app sidebar with library sections", "DEFAULT": "library", "BRAND": "Beats", "BRAND_CLR": "#ec4899", "SIDEBAR_BG": "#0f172a", "TEXT_CLR": "#94a3b8", "NAV1": "Library", "K1": "library", "NAV2": "Playlists", "K2": "playlists", "NAV3": "Artists", "K3": "artists", "NAV4": "Albums", "K4": "albums"}),
    ("cms", {"TITLE": "CMS Panel", "DESC": "Content management sidebar navigation", "DEFAULT": "posts", "BRAND": "CMS", "BRAND_CLR": "#7c3aed", "SIDEBAR_BG": "#faf5ff", "TEXT_CLR": "#6b21a8", "NAV1": "Posts", "K1": "posts", "NAV2": "Pages", "K2": "pages", "NAV3": "Media", "K3": "media", "NAV4": "Comments", "K4": "comments"}),
    ("analytics", {"TITLE": "Analytics Hub", "DESC": "Analytics dashboard sidebar navigation", "DEFAULT": "overview", "BRAND": "Analytics", "BRAND_CLR": "#0891b2", "SIDEBAR_BG": "#ecfeff", "TEXT_CLR": "#155e75", "NAV1": "Overview", "K1": "overview", "NAV2": "Traffic", "K2": "traffic", "NAV3": "Conversions", "K3": "conversions", "NAV4": "Audience", "K4": "audience"}),
    ("project", {"TITLE": "Project Manager", "DESC": "Project management sidebar with workflow tabs", "DEFAULT": "board", "BRAND": "Projects", "BRAND_CLR": "#f59e0b", "SIDEBAR_BG": "#fffbeb", "TEXT_CLR": "#92400e", "NAV1": "Board", "K1": "board", "NAV2": "Timeline", "K2": "timeline", "NAV3": "Backlog", "K3": "backlog", "NAV4": "Archive", "K4": "archive"}),
    ("chat", {"TITLE": "Chat App", "DESC": "Chat application sidebar with channel list", "DEFAULT": "general", "BRAND": "Chat", "BRAND_CLR": "#16a34a", "SIDEBAR_BG": "#f0fdf4", "TEXT_CLR": "#166534", "NAV1": "General", "K1": "general", "NAV2": "Random", "K2": "random", "NAV3": "Help", "K3": "help", "NAV4": "Announcements", "K4": "announcements"}),
    ("filemanager", {"TITLE": "File Manager", "DESC": "File manager sidebar with directory tree", "DEFAULT": "documents", "BRAND": "Files", "BRAND_CLR": "#1e293b", "SIDEBAR_BG": "#f1f5f9", "TEXT_CLR": "#475569", "NAV1": "Documents", "K1": "documents", "NAV2": "Images", "K2": "images", "NAV3": "Downloads", "K3": "downloads", "NAV4": "Shared", "K4": "shared"}),
    ("hr", {"TITLE": "HR Portal", "DESC": "HR portal sidebar with department navigation", "DEFAULT": "employees", "BRAND": "HR Hub", "BRAND_CLR": "#be185d", "SIDEBAR_BG": "#fce7f3", "TEXT_CLR": "#9d174d", "NAV1": "Employees", "K1": "employees", "NAV2": "Payroll", "K2": "payroll", "NAV3": "Leave", "K3": "leave", "NAV4": "Reviews", "K4": "reviews"}),
]:
    ex(f"gen-sidebar-{n}.naze", cfg["DESC"], fill(SIDEBAR_T, cfg))


# ─── 2. Modal dialogs (10) ───────────────────────────────────────────────────

MODAL_T = """-- __DESC__
app "__TITLE__" {
  state show-modal = false
  state confirmed = false

  column padding: 24px, gap: 16px {
    heading "__TITLE__"
    text "__INTRO__" color: #64748b

    rect width: 140px, height: 40px, color: __BTN_CLR__, radius: 8px {
      text "__BTN_TEXT__" color: #ffffff
      on click: set show-modal = true
    }

    if confirmed {
      rect padding: 12px, color: #f0fdf4, radius: 8px {
        text "__CONFIRM_MSG__" color: #16a34a
      }
    }

    if show-modal {
      rect width: 400px, height: 220px, color: #00000066, radius: 0px {
        rect width: 360px, height: 180px, color: #ffffff, radius: 12px {
          column padding: 24px, gap: 12px {
            text "__MODAL_TITLE__" font-weight: bold, font-size: 18px
            text "__MODAL_BODY__" color: #64748b
            row gap: 8px {
              rect width: 100px, height: 36px, color: __BTN_CLR__, radius: 6px {
                text "__YES__" color: #ffffff
                on click: set confirmed = true
                on click: set show-modal = false
              }
              rect width: 100px, height: 36px, color: #e2e8f0, radius: 6px {
                text "__NO__" color: #334155
                on click: set show-modal = false
              }
            }
          }
        }
      }
    }
  }
}"""

for n, cfg in [
    ("delete", {"TITLE": "Delete Confirm", "DESC": "Delete confirmation modal dialog", "INTRO": "Manage your items below.", "BTN_CLR": "#ef4444", "BTN_TEXT": "Delete Item", "CONFIRM_MSG": "Item deleted successfully.", "MODAL_TITLE": "Delete Item?", "MODAL_BODY": "This action cannot be undone. Are you sure?", "YES": "Delete", "NO": "Cancel"}),
    ("logout", {"TITLE": "Logout Confirm", "DESC": "Logout confirmation dialog", "INTRO": "You are currently signed in.", "BTN_CLR": "#64748b", "BTN_TEXT": "Sign Out", "CONFIRM_MSG": "You have been signed out.", "MODAL_TITLE": "Sign Out?", "MODAL_BODY": "Are you sure you want to sign out?", "YES": "Sign Out", "NO": "Stay"}),
    ("publish", {"TITLE": "Publish Confirm", "DESC": "Publish confirmation dialog for content", "INTRO": "Your article is ready for review.", "BTN_CLR": "#16a34a", "BTN_TEXT": "Publish Now", "CONFIRM_MSG": "Article published!", "MODAL_TITLE": "Publish Article?", "MODAL_BODY": "This will make your article visible to everyone.", "YES": "Publish", "NO": "Not Yet"}),
    ("discard", {"TITLE": "Discard Changes", "DESC": "Discard unsaved changes modal", "INTRO": "You have unsaved changes in your editor.", "BTN_CLR": "#f59e0b", "BTN_TEXT": "Discard", "CONFIRM_MSG": "Changes discarded.", "MODAL_TITLE": "Discard Changes?", "MODAL_BODY": "All unsaved changes will be lost.", "YES": "Discard", "NO": "Keep"}),
    ("subscribe", {"TITLE": "Subscribe Dialog", "DESC": "Newsletter subscription modal", "INTRO": "Stay up to date with our content.", "BTN_CLR": "#8b5cf6", "BTN_TEXT": "Subscribe", "CONFIRM_MSG": "You are now subscribed!", "MODAL_TITLE": "Subscribe?", "MODAL_BODY": "You will receive weekly email updates.", "YES": "Yes Please", "NO": "No Thanks"}),
    ("archive", {"TITLE": "Archive Confirm", "DESC": "Archive item confirmation modal", "INTRO": "Manage your project items.", "BTN_CLR": "#0891b2", "BTN_TEXT": "Archive", "CONFIRM_MSG": "Item archived.", "MODAL_TITLE": "Archive Item?", "MODAL_BODY": "Archived items can be restored later.", "YES": "Archive", "NO": "Cancel"}),
    ("reset", {"TITLE": "Reset Settings", "DESC": "Reset to defaults confirmation modal", "INTRO": "Your settings have been customized.", "BTN_CLR": "#dc2626", "BTN_TEXT": "Reset All", "CONFIRM_MSG": "Settings reset to defaults.", "MODAL_TITLE": "Reset Settings?", "MODAL_BODY": "All settings will return to their default values.", "YES": "Reset", "NO": "Keep"}),
    ("upgrade", {"TITLE": "Upgrade Plan", "DESC": "Plan upgrade confirmation modal", "INTRO": "You are on the free plan.", "BTN_CLR": "#7c3aed", "BTN_TEXT": "Upgrade", "CONFIRM_MSG": "Upgraded to Pro plan!", "MODAL_TITLE": "Upgrade to Pro?", "MODAL_BODY": "Get unlimited access for $9/month.", "YES": "Upgrade", "NO": "Maybe Later"}),
    ("leave", {"TITLE": "Leave Team", "DESC": "Leave team confirmation dialog", "INTRO": "You are a member of this team.", "BTN_CLR": "#e11d48", "BTN_TEXT": "Leave Team", "CONFIRM_MSG": "You have left the team.", "MODAL_TITLE": "Leave Team?", "MODAL_BODY": "You will lose access to team resources.", "YES": "Leave", "NO": "Stay"}),
    ("send", {"TITLE": "Send Message", "DESC": "Message send confirmation modal", "INTRO": "Your message is ready to send.", "BTN_CLR": "#2563eb", "BTN_TEXT": "Send", "CONFIRM_MSG": "Message sent!", "MODAL_TITLE": "Send Message?", "MODAL_BODY": "This message will be delivered to all recipients.", "YES": "Send", "NO": "Edit"}),
]:
    ex(f"gen-modal-{n}.naze", cfg["DESC"], fill(MODAL_T, cfg))


# ─── 3. Tab interfaces (12) ──────────────────────────────────────────────────

TAB_T = """-- __DESC__
app "__TITLE__" {
  state tab = "__DEFAULT__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    row gap: 0px {
      rect width: 100px, height: 36px, color: __C1__, radius: 0px {
        text "__TAB1__" color: __T1__
        on click: set tab = "__K1__"
      }
      rect width: 100px, height: 36px, color: __C2__, radius: 0px {
        text "__TAB2__" color: __T2__
        on click: set tab = "__K2__"
      }
      rect width: 100px, height: 36px, color: __C3__, radius: 0px {
        text "__TAB3__" color: __T3__
        on click: set tab = "__K3__"
      }
    }

    match tab {
      "__K1__": column padding: 16px, gap: 8px {
        text "__BODY1A__" font-weight: bold
        text "__BODY1B__" color: #64748b
      }
      "__K2__": column padding: 16px, gap: 8px {
        text "__BODY2A__" font-weight: bold
        text "__BODY2B__" color: #64748b
      }
      "__K3__": column padding: 16px, gap: 8px {
        text "__BODY3A__" font-weight: bold
        text "__BODY3B__" color: #64748b
      }
      _: text "Select a tab"
    }
  }
}"""

for n, cfg in [
    ("profile", {"TITLE": "User Profile", "DESC": "Profile tabs with info, activity, and settings", "DEFAULT": "info", "TAB1": "Info", "K1": "info", "TAB2": "Activity", "K2": "activity", "TAB3": "Settings", "K3": "settings", "C1": "#2563eb", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Personal Information", "BODY1B": "Name, email, and bio details.", "BODY2A": "Recent Activity", "BODY2B": "Posts, comments, and likes.", "BODY3A": "Account Settings", "BODY3B": "Password, notifications, privacy."}),
    ("product", {"TITLE": "Product Details", "DESC": "Product page tabs with description, specs, reviews", "DEFAULT": "desc", "TAB1": "Description", "K1": "desc", "TAB2": "Specs", "K2": "specs", "TAB3": "Reviews", "K3": "reviews", "C1": "#16a34a", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Product Overview", "BODY1B": "A premium quality item built for durability.", "BODY2A": "Technical Specifications", "BODY2B": "Weight: 250g, Dimensions: 15x10x5cm", "BODY3A": "Customer Reviews", "BODY3B": "4.5 out of 5 stars from 128 reviews."}),
    ("code", {"TITLE": "Code Editor", "DESC": "Code editor tabs for file switching", "DEFAULT": "html", "TAB1": "HTML", "K1": "html", "TAB2": "CSS", "K2": "css", "TAB3": "JS", "K3": "js", "C1": "#1e293b", "C2": "#334155", "C3": "#334155", "T1": "#e2e8f0", "T2": "#94a3b8", "T3": "#94a3b8", "BODY1A": "index.html", "BODY1B": "Main document structure and content.", "BODY2A": "styles.css", "BODY2B": "Layout, colors, and typography rules.", "BODY3A": "app.js", "BODY3B": "Event handlers and application logic."}),
    ("billing", {"TITLE": "Billing Center", "DESC": "Billing tabs with invoices, methods, and history", "DEFAULT": "invoices", "TAB1": "Invoices", "K1": "invoices", "TAB2": "Methods", "K2": "methods", "TAB3": "History", "K3": "history", "C1": "#7c3aed", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Current Invoices", "BODY1B": "No outstanding invoices at this time.", "BODY2A": "Payment Methods", "BODY2B": "Visa ending in 4242 is your default.", "BODY3A": "Transaction History", "BODY3B": "12 transactions in the last 30 days."}),
    ("course", {"TITLE": "Course Content", "DESC": "Course tabs with lessons, resources, and discussion", "DEFAULT": "lessons", "TAB1": "Lessons", "K1": "lessons", "TAB2": "Resources", "K2": "resources", "TAB3": "Discussion", "K3": "discussion", "C1": "#f59e0b", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Module 1: Getting Started", "BODY1B": "5 lessons, estimated 2 hours.", "BODY2A": "Downloadable Materials", "BODY2B": "Slides, cheat sheets, and exercises.", "BODY3A": "Student Discussion", "BODY3B": "23 threads, 5 new posts today."}),
    ("job", {"TITLE": "Job Listing", "DESC": "Job listing tabs with details, company, and apply", "DEFAULT": "details", "TAB1": "Details", "K1": "details", "TAB2": "Company", "K2": "company", "TAB3": "Apply", "K3": "apply", "C1": "#0891b2", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Senior Developer", "BODY1B": "Remote, full-time, competitive salary.", "BODY2A": "About the Company", "BODY2B": "A growing startup with 50 employees.", "BODY3A": "Application Form", "BODY3B": "Submit your resume and cover letter."}),
    ("recipe", {"TITLE": "Recipe Viewer", "DESC": "Recipe tabs with ingredients, steps, and nutrition", "DEFAULT": "ingredients", "TAB1": "Ingredients", "K1": "ingredients", "TAB2": "Steps", "K2": "steps", "TAB3": "Nutrition", "K3": "nutrition", "C1": "#ef4444", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Ingredients List", "BODY1B": "2 cups flour, 1 cup sugar, 3 eggs, butter.", "BODY2A": "Cooking Steps", "BODY2B": "Mix dry ingredients, add wet, bake at 350F.", "BODY3A": "Nutrition Facts", "BODY3B": "320 calories per serving, 12g protein."}),
    ("support", {"TITLE": "Support Center", "DESC": "Support center tabs with FAQ, tickets, and contact", "DEFAULT": "faq", "TAB1": "FAQ", "K1": "faq", "TAB2": "Tickets", "K2": "tickets", "TAB3": "Contact", "K3": "contact", "C1": "#ec4899", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Frequently Asked Questions", "BODY1B": "Browse common questions and answers.", "BODY2A": "My Support Tickets", "BODY2B": "2 open tickets, 5 resolved.", "BODY3A": "Contact Us", "BODY3B": "Email support@example.com for help."}),
    ("analytics-tab", {"TITLE": "Analytics View", "DESC": "Analytics tabs with overview, traffic, and conversions", "DEFAULT": "overview", "TAB1": "Overview", "K1": "overview", "TAB2": "Traffic", "K2": "traffic", "TAB3": "Goals", "K3": "goals", "C1": "#06b6d4", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Dashboard Overview", "BODY1B": "1,240 visitors today, up 12 percent from last week.", "BODY2A": "Traffic Sources", "BODY2B": "Direct 45 percent, Search 30 percent, Social 25 percent.", "BODY3A": "Goal Tracking", "BODY3B": "3 of 5 monthly goals completed."}),
    ("settings-tab", {"TITLE": "App Settings", "DESC": "Settings tabs with general, security, and notifications", "DEFAULT": "general", "TAB1": "General", "K1": "general", "TAB2": "Security", "K2": "security", "TAB3": "Alerts", "K3": "alerts", "C1": "#64748b", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "General Settings", "BODY1B": "Language, timezone, and display preferences.", "BODY2A": "Security Settings", "BODY2B": "Two-factor authentication and sessions.", "BODY3A": "Notification Preferences", "BODY3B": "Email, push, and in-app alert settings."}),
    ("portfolio", {"TITLE": "Portfolio", "DESC": "Portfolio tabs with work, about, and contact", "DEFAULT": "work", "TAB1": "Work", "K1": "work", "TAB2": "About", "K2": "about", "TAB3": "Contact", "K3": "contact", "C1": "#1e293b", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Selected Projects", "BODY1B": "Web apps, mobile apps, and design work.", "BODY2A": "About Me", "BODY2B": "Designer and developer with 8 years experience.", "BODY3A": "Get in Touch", "BODY3B": "Available for freelance work."}),
    ("travel", {"TITLE": "Trip Planner", "DESC": "Travel planning tabs with flights, hotels, and activities", "DEFAULT": "flights", "TAB1": "Flights", "K1": "flights", "TAB2": "Hotels", "K2": "hotels", "TAB3": "Activities", "K3": "activities", "C1": "#14b8a6", "C2": "#e2e8f0", "C3": "#e2e8f0", "T1": "#ffffff", "T2": "#334155", "T3": "#334155", "BODY1A": "Flight Search", "BODY1B": "Find the best deals on flights.", "BODY2A": "Hotel Booking", "BODY2B": "Compare hotels and guesthouses.", "BODY3A": "Local Activities", "BODY3B": "Tours, dining, and attractions nearby."}),
]:
    ex(f"gen-tab-{n}.naze", cfg["DESC"], fill(TAB_T, cfg))


# ─── 4. Accordion / collapsible sections (10) ────────────────────────────────

ACCORDION_T = """-- __DESC__
app "__TITLE__" {
  state open-section = ""

  column padding: 20px, gap: 8px {
    heading "__TITLE__"

    rect padding: 12px, color: __BG__, radius: 8px {
      text "__Q1__" font-weight: bold, color: __HDR_CLR__
      on click: set open-section = "s1"
    }
    if open-section == "s1" {
      rect padding: 12px, color: __CONTENT_BG__ {
        text "__A1__" color: #475569
      }
    }

    rect padding: 12px, color: __BG__, radius: 8px {
      text "__Q2__" font-weight: bold, color: __HDR_CLR__
      on click: set open-section = "s2"
    }
    if open-section == "s2" {
      rect padding: 12px, color: __CONTENT_BG__ {
        text "__A2__" color: #475569
      }
    }

    rect padding: 12px, color: __BG__, radius: 8px {
      text "__Q3__" font-weight: bold, color: __HDR_CLR__
      on click: set open-section = "s3"
    }
    if open-section == "s3" {
      rect padding: 12px, color: __CONTENT_BG__ {
        text "__A3__" color: #475569
      }
    }

    rect padding: 12px, color: __BG__, radius: 8px {
      text "__Q4__" font-weight: bold, color: __HDR_CLR__
      on click: set open-section = "s4"
    }
    if open-section == "s4" {
      rect padding: 12px, color: __CONTENT_BG__ {
        text "__A4__" color: #475569
      }
    }
  }
}"""

for n, cfg in [
    ("pricing-faq", {"TITLE": "Pricing FAQ", "DESC": "Pricing frequently asked questions accordion", "BG": "#f8fafc", "CONTENT_BG": "#f1f5f9", "HDR_CLR": "#1e293b", "Q1": "How much does it cost?", "A1": "Plans start at $9 per month for individuals.", "Q2": "Is there a free trial?", "A2": "Yes, enjoy a 14-day free trial on all plans.", "Q3": "Can I cancel anytime?", "A3": "Absolutely. Cancel anytime with no penalties.", "Q4": "Do you offer refunds?", "A4": "Full refund within 30 days of purchase."}),
    ("tech-faq", {"TITLE": "Technical FAQ", "DESC": "Technical support FAQ accordion", "BG": "#f0f9ff", "CONTENT_BG": "#e0f2fe", "HDR_CLR": "#0369a1", "Q1": "What browsers are supported?", "A1": "Chrome, Firefox, Safari, and Edge.", "Q2": "Is there an API?", "A2": "Yes, a REST API with full documentation.", "Q3": "How is data stored?", "A3": "Encrypted at rest with AES-256.", "Q4": "What about uptime?", "A4": "We guarantee 99.9 percent uptime SLA."}),
    ("legal-faq", {"TITLE": "Legal FAQ", "DESC": "Legal terms accordion with collapsible answers", "BG": "#fafafa", "CONTENT_BG": "#f5f5f5", "HDR_CLR": "#171717", "Q1": "What are the terms of service?", "A1": "Standard terms apply. See full document.", "Q2": "How is my data used?", "A2": "Only for service operation. Never sold.", "Q3": "Can I export my data?", "A3": "Yes, full data export available anytime.", "Q4": "What is your GDPR policy?", "A4": "We are fully GDPR compliant."}),
    ("onboarding", {"TITLE": "Getting Started", "DESC": "Onboarding steps accordion guide", "BG": "#ecfdf5", "CONTENT_BG": "#d1fae5", "HDR_CLR": "#065f46", "Q1": "Step 1: Create Your Account", "A1": "Sign up with your email or social login.", "Q2": "Step 2: Set Up Your Profile", "A2": "Add a photo, bio, and your interests.", "Q3": "Step 3: Connect Your Tools", "A3": "Link GitHub, Slack, or other services.", "Q4": "Step 4: Start Building", "A4": "Create your first project from templates."}),
    ("health-faq", {"TITLE": "Health FAQ", "DESC": "Health and wellness FAQ accordion", "BG": "#fff7ed", "CONTENT_BG": "#ffedd5", "HDR_CLR": "#9a3412", "Q1": "How many calories should I eat?", "A1": "2000 calories daily is a general guideline.", "Q2": "How much water should I drink?", "A2": "Aim for 8 glasses or about 2 liters daily.", "Q3": "How much sleep do I need?", "A3": "Adults need 7 to 9 hours per night.", "Q4": "How often should I exercise?", "A4": "At least 150 minutes of moderate exercise weekly."}),
    ("shipping-faq", {"TITLE": "Shipping FAQ", "DESC": "E-commerce shipping FAQ accordion", "BG": "#fef2f2", "CONTENT_BG": "#fee2e2", "HDR_CLR": "#991b1b", "Q1": "How long does shipping take?", "A1": "Standard shipping is 3 to 5 business days.", "Q2": "Do you ship internationally?", "A2": "Yes, to over 50 countries worldwide.", "Q3": "How do I track my order?", "A3": "A tracking link is emailed after dispatch.", "Q4": "What if my order is damaged?", "A4": "Contact us within 48 hours for a replacement."}),
    ("invest-faq", {"TITLE": "Investment FAQ", "DESC": "Investment platform FAQ accordion", "BG": "#f5f3ff", "CONTENT_BG": "#ede9fe", "HDR_CLR": "#5b21b6", "Q1": "What is the minimum investment?", "A1": "You can start with as little as $10.", "Q2": "How are returns calculated?", "A2": "Based on market performance, updated daily.", "Q3": "Are there any fees?", "A3": "0.5 percent annual management fee, no hidden costs.", "Q4": "Can I withdraw anytime?", "A4": "Yes, withdrawals are processed within 24 hours."}),
    ("edu-faq", {"TITLE": "Course FAQ", "DESC": "Online course FAQ accordion", "BG": "#fdf4ff", "CONTENT_BG": "#fae8ff", "HDR_CLR": "#86198f", "Q1": "Who is this course for?", "A1": "Beginners and intermediate learners.", "Q2": "How long is the course?", "A2": "12 weeks, 4 to 6 hours per week.", "Q3": "Do I get a certificate?", "A3": "Yes, a certificate upon completion.", "Q4": "Is there mentoring?", "A4": "Weekly live Q and A sessions with instructors."}),
    ("saas-faq", {"TITLE": "SaaS FAQ", "DESC": "SaaS product FAQ accordion", "BG": "#ecfeff", "CONTENT_BG": "#cffafe", "HDR_CLR": "#155e75", "Q1": "What platforms are supported?", "A1": "Web, iOS, Android, and desktop.", "Q2": "Is team collaboration supported?", "A2": "Yes, with real-time collaboration features.", "Q3": "How do integrations work?", "A3": "Connect via Zapier, webhooks, or our API.", "Q4": "What about data backup?", "A4": "Automatic daily backups with 30-day retention."}),
    ("rental-faq", {"TITLE": "Rental FAQ", "DESC": "Property rental FAQ accordion", "BG": "#fef3c7", "CONTENT_BG": "#fde68a", "HDR_CLR": "#92400e", "Q1": "How do I book a property?", "A1": "Browse listings and request a booking online.", "Q2": "What is the deposit policy?", "A2": "One month deposit, refundable at checkout.", "Q3": "Are pets allowed?", "A3": "Depends on the property. Check each listing.", "Q4": "How do I report an issue?", "A4": "Use the in-app maintenance request form."}),
]:
    ex(f"gen-accordion-{n}.naze", cfg["DESC"], fill(ACCORDION_T, cfg))


# ─── 5. Toast notifications / snackbars (8) ──────────────────────────────────

TOAST_T = """-- __DESC__
app "__TITLE__" {
  state show-toast = false
  state action-done = false

  column padding: 24px, gap: 16px {
    heading "__TITLE__"
    text "__INTRO__" color: #64748b

    row gap: 8px {
      rect width: 120px, height: 40px, color: __BTN_CLR__, radius: 8px {
        text "__BTN__" color: #ffffff
        on click: set show-toast = true
        on click: set action-done = true
      }
    }

    if show-toast {
      rect width: 340px, height: 48px, color: __TOAST_BG__, radius: 8px {
        row padding: 12px, gap: 12px {
          text "__TOAST_ICON__" font-size: 16px
          text "__TOAST_MSG__" color: __TOAST_CLR__, font-size: 14px
          spacer
          rect width: 24px, height: 24px, radius: 4px {
            text "x" color: __TOAST_CLR__, font-size: 12px
            on click: set show-toast = false
          }
        }
      }
    }
  }
}"""

for n, cfg in [
    ("success", {"TITLE": "Save Success", "DESC": "Success toast after saving data", "INTRO": "Edit your profile settings below.", "BTN_CLR": "#16a34a", "BTN": "Save", "TOAST_BG": "#f0fdf4", "TOAST_CLR": "#166534", "TOAST_ICON": "[OK]", "TOAST_MSG": "Changes saved successfully!"}),
    ("error", {"TITLE": "Error Alert", "DESC": "Error toast notification on failed action", "INTRO": "Submit the form to proceed.", "BTN_CLR": "#ef4444", "BTN": "Submit", "TOAST_BG": "#fef2f2", "TOAST_CLR": "#991b1b", "TOAST_ICON": "[!]", "TOAST_MSG": "Something went wrong. Try again."}),
    ("warning", {"TITLE": "Warning Notice", "DESC": "Warning toast for risky operations", "INTRO": "You are about to make changes.", "BTN_CLR": "#f59e0b", "BTN": "Proceed", "TOAST_BG": "#fffbeb", "TOAST_CLR": "#92400e", "TOAST_ICON": "[!!]", "TOAST_MSG": "This action may affect other users."}),
    ("info", {"TITLE": "Info Banner", "DESC": "Informational toast notification", "INTRO": "Check for updates regularly.", "BTN_CLR": "#2563eb", "BTN": "Check", "TOAST_BG": "#eff6ff", "TOAST_CLR": "#1e40af", "TOAST_ICON": "[i]", "TOAST_MSG": "A new version is available."}),
    ("copied", {"TITLE": "Copy Feedback", "DESC": "Toast confirmation after copying text", "INTRO": "Click the button to copy the link.", "BTN_CLR": "#0891b2", "BTN": "Copy Link", "TOAST_BG": "#ecfeff", "TOAST_CLR": "#155e75", "TOAST_ICON": "[+]", "TOAST_MSG": "Link copied to clipboard!"}),
    ("undo", {"TITLE": "Undo Action", "DESC": "Toast with undo action after deletion", "INTRO": "Manage your items in the list.", "BTN_CLR": "#64748b", "BTN": "Delete", "TOAST_BG": "#1e293b", "TOAST_CLR": "#e2e8f0", "TOAST_ICON": "[-]", "TOAST_MSG": "Item deleted. Undo?"}),
    ("added", {"TITLE": "Cart Toast", "DESC": "Toast after adding item to cart", "INTRO": "Browse our product catalog.", "BTN_CLR": "#7c3aed", "BTN": "Add to Cart", "TOAST_BG": "#f5f3ff", "TOAST_CLR": "#5b21b6", "TOAST_ICON": "[+]", "TOAST_MSG": "Item added to your cart!"}),
    ("welcome", {"TITLE": "Welcome Toast", "DESC": "Welcome toast for new users", "INTRO": "You have just signed up.", "BTN_CLR": "#ec4899", "BTN": "Get Started", "TOAST_BG": "#fdf2f8", "TOAST_CLR": "#9d174d", "TOAST_ICON": "[*]", "TOAST_MSG": "Welcome! Let us set up your profile."}),
]:
    ex(f"gen-toast-{n}.naze", cfg["DESC"], fill(TOAST_T, cfg))


# ─── 6. Avatar displays (10) ─────────────────────────────────────────────────

AVATAR_T = """-- __DESC__
app "__TITLE__" {
  column padding: 24px, gap: 16px {
    heading "__TITLE__"

    row gap: 16px {
      column gap: 4px {
        rect width: 48px, height: 48px, color: __C1__, radius: 24px {
          text "__I1__" color: #ffffff, font-size: 18px, font-weight: bold
        }
        text "__N1__" font-size: 12px, color: #64748b
      }
      column gap: 4px {
        rect width: 48px, height: 48px, color: __C2__, radius: 24px {
          text "__I2__" color: #ffffff, font-size: 18px, font-weight: bold
        }
        text "__N2__" font-size: 12px, color: #64748b
      }
      column gap: 4px {
        rect width: 48px, height: 48px, color: __C3__, radius: 24px {
          text "__I3__" color: #ffffff, font-size: 18px, font-weight: bold
        }
        text "__N3__" font-size: 12px, color: #64748b
      }
      column gap: 4px {
        rect width: 48px, height: 48px, color: __C4__, radius: 24px {
          text "__I4__" color: #ffffff, font-size: 18px, font-weight: bold
        }
        text "__N4__" font-size: 12px, color: #64748b
      }
    }

    text "__SUBTITLE__" color: #94a3b8, font-size: 14px
  }
}"""

for n, cfg in [
    ("team", {"TITLE": "Team Members", "DESC": "Team member avatars with initials", "C1": "#3b82f6", "C2": "#ef4444", "C3": "#16a34a", "C4": "#f59e0b", "I1": "AB", "N1": "Alice B.", "I2": "CD", "N2": "Charlie D.", "I3": "EF", "N3": "Eve F.", "I4": "GH", "N4": "Grace H.", "SUBTITLE": "4 team members online"}),
    ("reviewers", {"TITLE": "Reviewers", "DESC": "Code reviewer avatar badges", "C1": "#8b5cf6", "C2": "#0891b2", "C3": "#e11d48", "C4": "#84cc16", "I1": "JK", "N1": "Jake K.", "I2": "LM", "N2": "Luna M.", "I3": "NO", "N3": "Nora O.", "I4": "PQ", "N4": "Paul Q.", "SUBTITLE": "Assigned reviewers for this PR"}),
    ("contacts", {"TITLE": "Contacts", "DESC": "Favorite contacts avatar list", "C1": "#dc2626", "C2": "#2563eb", "C3": "#7c3aed", "C4": "#059669", "I1": "RS", "N1": "Rosa S.", "I2": "TU", "N2": "Tom U.", "I3": "VW", "N3": "Vera W.", "I4": "XY", "N4": "Xena Y.", "SUBTITLE": "Your favorite contacts"}),
    ("students", {"TITLE": "Study Group", "DESC": "Study group member avatars", "C1": "#f97316", "C2": "#14b8a6", "C3": "#6366f1", "C4": "#ec4899", "I1": "MJ", "N1": "Maya J.", "I2": "SK", "N2": "Sam K.", "I3": "LR", "N3": "Leo R.", "I4": "DW", "N4": "Dina W.", "SUBTITLE": "Members of Calculus 201 group"}),
    ("speakers", {"TITLE": "Speakers", "DESC": "Conference speaker avatars", "C1": "#1e293b", "C2": "#0369a1", "C3": "#b91c1c", "C4": "#4338ca", "I1": "DR", "N1": "Dr. Reid", "I2": "PL", "N2": "Prof. Lin", "I3": "MC", "N3": "Dr. Cruz", "I4": "AK", "N4": "Prof. Kim", "SUBTITLE": "Featured speakers at DevConf 2026"}),
    ("players", {"TITLE": "Game Lobby", "DESC": "Multiplayer game lobby player avatars", "C1": "#ef4444", "C2": "#3b82f6", "C3": "#eab308", "C4": "#22c55e", "I1": "P1", "N1": "Player 1", "I2": "P2", "N2": "Player 2", "I3": "P3", "N3": "Player 3", "I4": "P4", "N4": "Player 4", "SUBTITLE": "4 of 4 players ready"}),
    ("contributors", {"TITLE": "Contributors", "DESC": "Open source contributor avatars", "C1": "#0f172a", "C2": "#7c3aed", "C3": "#06b6d4", "C4": "#f43f5e", "I1": "KL", "N1": "Kai L.", "I2": "NP", "N2": "Nia P.", "I3": "OQ", "N3": "Omar Q.", "I4": "RD", "N4": "Rita D.", "SUBTITLE": "Top contributors this month"}),
    ("family", {"TITLE": "Family Group", "DESC": "Family shared album member avatars", "C1": "#ec4899", "C2": "#3b82f6", "C3": "#f59e0b", "C4": "#8b5cf6", "I1": "MA", "N1": "Mom", "I2": "DA", "N2": "Dad", "I3": "SI", "N3": "Sis", "I4": "BR", "N4": "Bro", "SUBTITLE": "Family photo album members"}),
    ("mentors", {"TITLE": "Mentors", "DESC": "Available mentors with initials avatars", "C1": "#059669", "C2": "#7c3aed", "C3": "#dc2626", "C4": "#0891b2", "I1": "JW", "N1": "Dr. Wells", "I2": "AT", "N2": "Prof. Tan", "I3": "BH", "N3": "Dr. Hart", "I4": "CF", "N4": "Prof. Fox", "SUBTITLE": "Available mentors for booking"}),
    ("band", {"TITLE": "Band Members", "DESC": "Music band member profile avatars", "C1": "#1e293b", "C2": "#ef4444", "C3": "#eab308", "C4": "#6366f1", "I1": "VO", "N1": "Vocals", "I2": "GT", "N2": "Guitar", "I3": "BS", "N3": "Bass", "I4": "DR", "N4": "Drums", "SUBTITLE": "The Midnight Signal"}),
]:
    ex(f"gen-avatar-{n}.naze", cfg["DESC"], fill(AVATAR_T, cfg))


# ─── 7. Tag lists / badges / chip groups (10) ────────────────────────────────

TAG_T = """-- __DESC__
app "__TITLE__" {
  state selected-tag = ""

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__INTRO__" color: #64748b

    row gap: 8px {
      rect padding: 8px, color: __C1__, radius: 12px {
        text "__TAG1__" color: __T1__, font-size: 13px
        on click: set selected-tag = "__TAG1__"
      }
      rect padding: 8px, color: __C2__, radius: 12px {
        text "__TAG2__" color: __T2__, font-size: 13px
        on click: set selected-tag = "__TAG2__"
      }
      rect padding: 8px, color: __C3__, radius: 12px {
        text "__TAG3__" color: __T3__, font-size: 13px
        on click: set selected-tag = "__TAG3__"
      }
      rect padding: 8px, color: __C4__, radius: 12px {
        text "__TAG4__" color: __T4__, font-size: 13px
        on click: set selected-tag = "__TAG4__"
      }
      rect padding: 8px, color: __C5__, radius: 12px {
        text "__TAG5__" color: __T5__, font-size: 13px
        on click: set selected-tag = "__TAG5__"
      }
    }

    if selected-tag {
      text "Selected: {selected-tag}" font-weight: bold, color: #334155
    }
  }
}"""

for n, cfg in [
    ("skills", {"TITLE": "Skills", "DESC": "Developer skill tags with selection", "INTRO": "Select your skills", "TAG1": "Python", "TAG2": "Rust", "TAG3": "TypeScript", "TAG4": "Go", "TAG5": "SQL", "C1": "#dbeafe", "T1": "#1e40af", "C2": "#fce7f3", "T2": "#9d174d", "C3": "#d1fae5", "T3": "#065f46", "C4": "#e0f2fe", "T4": "#0369a1", "C5": "#fef3c7", "T5": "#92400e"}),
    ("genres", {"TITLE": "Music Genres", "DESC": "Music genre filter tag chips", "INTRO": "Filter by genre", "TAG1": "Rock", "TAG2": "Jazz", "TAG3": "Electronic", "TAG4": "Classical", "TAG5": "Hip Hop", "C1": "#fef2f2", "T1": "#991b1b", "C2": "#faf5ff", "T2": "#6b21a8", "C3": "#ecfeff", "T3": "#155e75", "C4": "#fefce8", "T4": "#713f12", "C5": "#f0fdf4", "T5": "#14532d"}),
    ("categories", {"TITLE": "Blog Categories", "DESC": "Blog post category tags", "INTRO": "Browse by category", "TAG1": "Tutorial", "TAG2": "News", "TAG3": "Opinion", "TAG4": "Review", "TAG5": "Guide", "C1": "#eff6ff", "T1": "#1d4ed8", "C2": "#fdf2f8", "T2": "#be185d", "C3": "#f0fdf4", "T3": "#15803d", "C4": "#fff7ed", "T4": "#c2410c", "C5": "#f5f3ff", "T5": "#6d28d9"}),
    ("status", {"TITLE": "Task Status", "DESC": "Task status badge labels", "INTRO": "Filter by status", "TAG1": "Open", "TAG2": "In Progress", "TAG3": "Review", "TAG4": "Done", "TAG5": "Blocked", "C1": "#dbeafe", "T1": "#1e40af", "C2": "#fef3c7", "T2": "#92400e", "C3": "#f3e8ff", "T3": "#7e22ce", "C4": "#d1fae5", "T4": "#065f46", "C5": "#fee2e2", "T5": "#991b1b"}),
    ("dietary", {"TITLE": "Dietary Tags", "DESC": "Recipe dietary filter tags", "INTRO": "Filter recipes by diet", "TAG1": "Vegan", "TAG2": "Gluten-Free", "TAG3": "Keto", "TAG4": "Paleo", "TAG5": "Dairy-Free", "C1": "#d1fae5", "T1": "#065f46", "C2": "#fef3c7", "T2": "#92400e", "C3": "#fce7f3", "T3": "#9d174d", "C4": "#e0f2fe", "T4": "#0369a1", "C5": "#fef2f2", "T5": "#991b1b"}),
    ("priority-tags", {"TITLE": "Priority Labels", "DESC": "Issue priority label tags", "INTRO": "Assign priority", "TAG1": "Critical", "TAG2": "High", "TAG3": "Medium", "TAG4": "Low", "TAG5": "Trivial", "C1": "#fee2e2", "T1": "#991b1b", "C2": "#fff7ed", "T2": "#c2410c", "C3": "#fef3c7", "T3": "#92400e", "C4": "#dbeafe", "T4": "#1e40af", "C5": "#f1f5f9", "T5": "#475569"}),
    ("topics", {"TITLE": "Topic Tags", "DESC": "Forum topic tags for discussions", "INTRO": "Browse topics", "TAG1": "Help", "TAG2": "Showcase", "TAG3": "Bug Report", "TAG4": "Feature", "TAG5": "Discussion", "C1": "#ecfdf5", "T1": "#047857", "C2": "#f5f3ff", "T2": "#6d28d9", "C3": "#fef2f2", "T3": "#b91c1c", "C4": "#eff6ff", "T4": "#1d4ed8", "C5": "#f8fafc", "T5": "#334155"}),
    ("moods", {"TITLE": "Mood Tags", "DESC": "Playlist mood filter tags", "INTRO": "Set the mood", "TAG1": "Chill", "TAG2": "Energetic", "TAG3": "Focus", "TAG4": "Happy", "TAG5": "Melancholy", "C1": "#e0f2fe", "T1": "#0369a1", "C2": "#fef2f2", "T2": "#dc2626", "C3": "#f0fdf4", "T3": "#166534", "C4": "#fef3c7", "T4": "#b45309", "C5": "#f5f3ff", "T5": "#7c3aed"}),
    ("sizes", {"TITLE": "Size Selector", "DESC": "Clothing size badge chips", "INTRO": "Pick your size", "TAG1": "XS", "TAG2": "S", "TAG3": "M", "TAG4": "L", "TAG5": "XL", "C1": "#f1f5f9", "T1": "#475569", "C2": "#f1f5f9", "T2": "#475569", "C3": "#1e293b", "T3": "#f8fafc", "C4": "#f1f5f9", "T4": "#475569", "C5": "#f1f5f9", "T5": "#475569"}),
    ("labels", {"TITLE": "Email Labels", "DESC": "Email label tag management", "INTRO": "Organize with labels", "TAG1": "Work", "TAG2": "Personal", "TAG3": "Finance", "TAG4": "Travel", "TAG5": "Urgent", "C1": "#dbeafe", "T1": "#1d4ed8", "C2": "#d1fae5", "T2": "#15803d", "C3": "#fef3c7", "T3": "#a16207", "C4": "#fce7f3", "T4": "#be185d", "C5": "#fee2e2", "T5": "#b91c1c"}),
]:
    ex(f"gen-tag-{n}.naze", cfg["DESC"], fill(TAG_T, cfg))


# ─── 8. Empty state pages (10) ────────────────────────────────────────────────

EMPTY_T = """-- __DESC__
app "__TITLE__" {
  column padding: 40px, gap: 20px {
    spacer height: 40px

    rect width: 80px, height: 80px, color: __ICON_BG__, radius: 40px {
      text "__ICON__" font-size: 32px, color: __ICON_CLR__
    }

    heading "__HEADLINE__" font-size: 22px, color: #1e293b
    text "__SUBTEXT__" color: #64748b, font-size: 16px
    text "__HINT__" color: #94a3b8, font-size: 14px

    rect width: 160px, height: 44px, color: __BTN_CLR__, radius: 8px {
      text "__BTN__" color: #ffffff, font-size: 15px
      on click: navigate "__LINK__"
    }
  }
}"""

for n, cfg in [
    ("no-results", {"TITLE": "No Results", "DESC": "Empty search results state page", "ICON_BG": "#f1f5f9", "ICON_CLR": "#94a3b8", "ICON": "?", "HEADLINE": "No Results Found", "SUBTEXT": "We could not find anything matching your search.", "HINT": "Try different keywords or filters.", "BTN_CLR": "#2563eb", "BTN": "Clear Search", "LINK": "/search"}),
    ("no-messages", {"TITLE": "No Messages", "DESC": "Empty inbox state with no messages", "ICON_BG": "#eff6ff", "ICON_CLR": "#3b82f6", "ICON": "@", "HEADLINE": "No Messages Yet", "SUBTEXT": "Your inbox is empty.", "HINT": "Start a conversation with someone.", "BTN_CLR": "#3b82f6", "BTN": "Compose", "LINK": "/compose"}),
    ("no-projects", {"TITLE": "No Projects", "DESC": "Empty project list state page", "ICON_BG": "#f5f3ff", "ICON_CLR": "#7c3aed", "ICON": "+", "HEADLINE": "No Projects Yet", "SUBTEXT": "Create your first project to get started.", "HINT": "Projects help organize your work.", "BTN_CLR": "#7c3aed", "BTN": "New Project", "LINK": "/projects/new"}),
    ("no-tasks", {"TITLE": "All Done", "DESC": "Empty task list celebrating completion", "ICON_BG": "#f0fdf4", "ICON_CLR": "#16a34a", "ICON": "*", "HEADLINE": "All Tasks Complete!", "SUBTEXT": "You have finished everything on your list.", "HINT": "Enjoy your free time.", "BTN_CLR": "#16a34a", "BTN": "Add New Task", "LINK": "/tasks/new"}),
    ("no-orders", {"TITLE": "No Orders", "DESC": "Empty order history state", "ICON_BG": "#fff7ed", "ICON_CLR": "#f97316", "ICON": "#", "HEADLINE": "No Orders Yet", "SUBTEXT": "You have not placed any orders.", "HINT": "Browse our catalog to find something you like.", "BTN_CLR": "#f97316", "BTN": "Start Shopping", "LINK": "/shop"}),
    ("no-files", {"TITLE": "No Files", "DESC": "Empty file storage state page", "ICON_BG": "#ecfeff", "ICON_CLR": "#0891b2", "ICON": "^", "HEADLINE": "No Files Uploaded", "SUBTEXT": "Upload your first file to get started.", "HINT": "Drag and drop or click to upload.", "BTN_CLR": "#0891b2", "BTN": "Upload File", "LINK": "/upload"}),
    ("no-events", {"TITLE": "No Events", "DESC": "Empty calendar events state", "ICON_BG": "#fce7f3", "ICON_CLR": "#ec4899", "ICON": "~", "HEADLINE": "No Upcoming Events", "SUBTEXT": "Your calendar is clear.", "HINT": "Schedule an event to stay organized.", "BTN_CLR": "#ec4899", "BTN": "Create Event", "LINK": "/events/new"}),
    ("no-reviews", {"TITLE": "No Reviews", "DESC": "Empty reviews state for a product", "ICON_BG": "#fef3c7", "ICON_CLR": "#f59e0b", "ICON": "*", "HEADLINE": "No Reviews Yet", "SUBTEXT": "Be the first to share your experience.", "HINT": "Your feedback helps other customers.", "BTN_CLR": "#f59e0b", "BTN": "Write Review", "LINK": "/review/new"}),
    ("no-friends", {"TITLE": "No Connections", "DESC": "Empty friends or connections list state", "ICON_BG": "#fef2f2", "ICON_CLR": "#ef4444", "ICON": "&", "HEADLINE": "No Connections Yet", "SUBTEXT": "Find people to connect with.", "HINT": "Search by name or browse suggestions.", "BTN_CLR": "#ef4444", "BTN": "Find People", "LINK": "/discover"}),
    ("no-bookmarks", {"TITLE": "No Bookmarks", "DESC": "Empty bookmarks collection state", "ICON_BG": "#f1f5f9", "ICON_CLR": "#64748b", "ICON": ">", "HEADLINE": "No Bookmarks Saved", "SUBTEXT": "Save articles and pages for later reading.", "HINT": "Tap the bookmark icon on any article.", "BTN_CLR": "#64748b", "BTN": "Browse Articles", "LINK": "/articles"}),
]:
    ex(f"gen-empty-{n}.naze", cfg["DESC"], fill(EMPTY_T, cfg))


# ─── 9. Hero sections (10) ───────────────────────────────────────────────────

HERO_T = """-- __DESC__
app "__TITLE__" {
  column gap: 0px {
    rect width: 800px, height: 360px, color: __BG_CLR__ {
      column padding: 48px, gap: 16px {
        spacer height: 20px
        heading "__HEADLINE__" font-size: 36px, color: __HEAD_CLR__
        text "__SUBTEXT__" font-size: 18px, color: __SUB_CLR__

        row gap: 12px {
          rect width: 160px, height: 48px, color: __CTA1_CLR__, radius: 8px {
            text "__CTA1__" color: __CTA1_TEXT__, font-size: 16px
            on click: navigate "__CTA1_LINK__"
          }
          rect width: 140px, height: 48px, color: __CTA2_CLR__, radius: 8px {
            text "__CTA2__" color: __CTA2_TEXT__, font-size: 16px
            on click: navigate "__CTA2_LINK__"
          }
        }
      }
    }

    rect width: 800px, height: 48px, color: __FOOTER_BG__ {
      row padding: 12px, gap: 24px {
        text "__STAT1__" color: __FOOTER_CLR__, font-size: 14px
        text "__STAT2__" color: __FOOTER_CLR__, font-size: 14px
        text "__STAT3__" color: __FOOTER_CLR__, font-size: 14px
      }
    }
  }
}"""

for n, cfg in [
    ("saas", {"TITLE": "SaaS Landing", "DESC": "SaaS product hero section with CTAs", "BG_CLR": "#0f172a", "HEAD_CLR": "#f8fafc", "HEADLINE": "Ship Faster with Acme", "SUB_CLR": "#94a3b8", "SUBTEXT": "The all-in-one platform for modern teams.", "CTA1_CLR": "#2563eb", "CTA1_TEXT": "#ffffff", "CTA1": "Get Started", "CTA1_LINK": "/signup", "CTA2_CLR": "#1e293b", "CTA2_TEXT": "#e2e8f0", "CTA2": "Learn More", "CTA2_LINK": "/features", "FOOTER_BG": "#1e293b", "FOOTER_CLR": "#94a3b8", "STAT1": "10K+ users", "STAT2": "99.9 percent uptime", "STAT3": "24/7 support"}),
    ("portfolio-hero", {"TITLE": "Portfolio Hero", "DESC": "Creative portfolio hero section", "BG_CLR": "#fafafa", "HEAD_CLR": "#0f172a", "HEADLINE": "Hello, I am Alex", "SUB_CLR": "#64748b", "SUBTEXT": "Designer and developer crafting digital experiences.", "CTA1_CLR": "#0f172a", "CTA1_TEXT": "#ffffff", "CTA1": "View Work", "CTA1_LINK": "/portfolio", "CTA2_CLR": "#e2e8f0", "CTA2_TEXT": "#0f172a", "CTA2": "Contact Me", "CTA2_LINK": "/contact", "FOOTER_BG": "#f1f5f9", "FOOTER_CLR": "#64748b", "STAT1": "50+ projects", "STAT2": "8 years exp", "STAT3": "12 awards"}),
    ("startup", {"TITLE": "Startup Launch", "DESC": "Startup launch hero with signup CTA", "BG_CLR": "#4f46e5", "HEAD_CLR": "#ffffff", "HEADLINE": "Revolutionize Your Workflow", "SUB_CLR": "#c7d2fe", "SUBTEXT": "AI-powered tools that save you hours every day.", "CTA1_CLR": "#ffffff", "CTA1_TEXT": "#4f46e5", "CTA1": "Try Free", "CTA1_LINK": "/trial", "CTA2_CLR": "#6366f1", "CTA2_TEXT": "#ffffff", "CTA2": "Watch Demo", "CTA2_LINK": "/demo", "FOOTER_BG": "#4338ca", "FOOTER_CLR": "#c7d2fe", "STAT1": "500+ companies", "STAT2": "2M tasks done", "STAT3": "4.9 rating"}),
    ("ecommerce", {"TITLE": "Shop Hero", "DESC": "E-commerce store hero with seasonal sale", "BG_CLR": "#fef2f2", "HEAD_CLR": "#991b1b", "HEADLINE": "Spring Sale: 40 Percent Off", "SUB_CLR": "#dc2626", "SUBTEXT": "Limited time offer on all premium collections.", "CTA1_CLR": "#dc2626", "CTA1_TEXT": "#ffffff", "CTA1": "Shop Now", "CTA1_LINK": "/sale", "CTA2_CLR": "#fecaca", "CTA2_TEXT": "#991b1b", "CTA2": "View Catalog", "CTA2_LINK": "/catalog", "FOOTER_BG": "#fee2e2", "FOOTER_CLR": "#b91c1c", "STAT1": "Free shipping", "STAT2": "Easy returns", "STAT3": "Secure pay"}),
    ("education", {"TITLE": "Learn Hub", "DESC": "Education platform hero section", "BG_CLR": "#ecfdf5", "HEAD_CLR": "#065f46", "HEADLINE": "Learn Without Limits", "SUB_CLR": "#047857", "SUBTEXT": "Thousands of courses from world-class instructors.", "CTA1_CLR": "#059669", "CTA1_TEXT": "#ffffff", "CTA1": "Browse Courses", "CTA1_LINK": "/courses", "CTA2_CLR": "#d1fae5", "CTA2_TEXT": "#065f46", "CTA2": "Free Trial", "CTA2_LINK": "/trial", "FOOTER_BG": "#d1fae5", "FOOTER_CLR": "#065f46", "STAT1": "5K+ courses", "STAT2": "200K students", "STAT3": "95 percent rated 4+"}),
    ("devtool", {"TITLE": "Dev Tool Hero", "DESC": "Developer tool landing page hero", "BG_CLR": "#1e293b", "HEAD_CLR": "#38bdf8", "HEADLINE": "Build Better, Faster", "SUB_CLR": "#94a3b8", "SUBTEXT": "The developer toolkit trusted by top engineering teams.", "CTA1_CLR": "#0ea5e9", "CTA1_TEXT": "#ffffff", "CTA1": "Install Now", "CTA1_LINK": "/install", "CTA2_CLR": "#334155", "CTA2_TEXT": "#e2e8f0", "CTA2": "Read Docs", "CTA2_LINK": "/docs", "FOOTER_BG": "#0f172a", "FOOTER_CLR": "#64748b", "STAT1": "Open source", "STAT2": "15K stars", "STAT3": "MIT license"}),
    ("blog-hero", {"TITLE": "Blog Hero", "DESC": "Blog landing hero with latest post CTA", "BG_CLR": "#faf5ff", "HEAD_CLR": "#581c87", "HEADLINE": "Stories Worth Reading", "SUB_CLR": "#7c3aed", "SUBTEXT": "Insights on technology, design, and culture.", "CTA1_CLR": "#7c3aed", "CTA1_TEXT": "#ffffff", "CTA1": "Latest Posts", "CTA1_LINK": "/blog", "CTA2_CLR": "#ede9fe", "CTA2_TEXT": "#6d28d9", "CTA2": "Subscribe", "CTA2_LINK": "/subscribe", "FOOTER_BG": "#ede9fe", "FOOTER_CLR": "#7c3aed", "STAT1": "500+ articles", "STAT2": "50K readers", "STAT3": "Weekly digest"}),
    ("fitness", {"TITLE": "Fitness Hero", "DESC": "Fitness app hero section with workout CTA", "BG_CLR": "#0f172a", "HEAD_CLR": "#22c55e", "HEADLINE": "Train Smarter Today", "SUB_CLR": "#86efac", "SUBTEXT": "Personalized workouts and nutrition plans.", "CTA1_CLR": "#22c55e", "CTA1_TEXT": "#0f172a", "CTA1": "Start Training", "CTA1_LINK": "/workouts", "CTA2_CLR": "#1e293b", "CTA2_TEXT": "#86efac", "CTA2": "View Plans", "CTA2_LINK": "/plans", "FOOTER_BG": "#1e293b", "FOOTER_CLR": "#4ade80", "STAT1": "100+ workouts", "STAT2": "AI coaching", "STAT3": "Free plan"}),
    ("nonprofit", {"TITLE": "Charity Hero", "DESC": "Nonprofit donation hero section", "BG_CLR": "#fffbeb", "HEAD_CLR": "#92400e", "HEADLINE": "Make a Difference", "SUB_CLR": "#b45309", "SUBTEXT": "Every donation helps build a better future.", "CTA1_CLR": "#f59e0b", "CTA1_TEXT": "#ffffff", "CTA1": "Donate Now", "CTA1_LINK": "/donate", "CTA2_CLR": "#fef3c7", "CTA2_TEXT": "#92400e", "CTA2": "Our Mission", "CTA2_LINK": "/about", "FOOTER_BG": "#fef3c7", "FOOTER_CLR": "#92400e", "STAT1": "$2M raised", "STAT2": "50K donors", "STAT3": "30 countries"}),
    ("music-hero", {"TITLE": "Music Hero", "DESC": "Music streaming platform hero section", "BG_CLR": "#18181b", "HEAD_CLR": "#f472b6", "HEADLINE": "Your Music, Your Way", "SUB_CLR": "#a1a1aa", "SUBTEXT": "Stream millions of songs ad-free.", "CTA1_CLR": "#ec4899", "CTA1_TEXT": "#ffffff", "CTA1": "Start Listening", "CTA1_LINK": "/listen", "CTA2_CLR": "#27272a", "CTA2_TEXT": "#f472b6", "CTA2": "View Plans", "CTA2_LINK": "/pricing", "FOOTER_BG": "#27272a", "FOOTER_CLR": "#a1a1aa", "STAT1": "80M+ songs", "STAT2": "No ads", "STAT3": "Hi-fi audio"}),
]:
    ex(f"gen-hero-{n}.naze", cfg["DESC"], fill(HERO_T, cfg))


# ─── 10. Toolbar / action bar patterns (10) ──────────────────────────────────

TOOLBAR_T = """-- __DESC__
app "__TITLE__" {
  state active-tool = "__DEFAULT__"

  column gap: 0px {
    rect width: 600px, height: 52px, color: __BAR_BG__ {
      row padding: 8px, gap: 6px {
        rect width: 36px, height: 36px, color: __IC1__, radius: 6px {
          text "__B1__" color: __BTN_TEXT__, font-size: 14px
          on click: set active-tool = "__K1__"
        }
        rect width: 36px, height: 36px, color: __IC2__, radius: 6px {
          text "__B2__" color: __BTN_TEXT__, font-size: 14px
          on click: set active-tool = "__K2__"
        }
        rect width: 36px, height: 36px, color: __IC3__, radius: 6px {
          text "__B3__" color: __BTN_TEXT__, font-size: 14px
          on click: set active-tool = "__K3__"
        }
        rect width: 36px, height: 36px, color: __IC4__, radius: 6px {
          text "__B4__" color: __BTN_TEXT__, font-size: 14px
          on click: set active-tool = "__K4__"
        }
        spacer
        rect width: 36px, height: 36px, color: __IC5__, radius: 6px {
          text "__B5__" color: __BTN_TEXT__, font-size: 14px
          on click: set active-tool = "__K5__"
        }
      }
    }

    column padding: 20px, gap: 8px {
      text "Tool: {active-tool}" font-weight: bold, font-size: 16px
      match active-tool {
        "__K1__": text "__D1__" color: #64748b
        "__K2__": text "__D2__" color: #64748b
        "__K3__": text "__D3__" color: #64748b
        "__K4__": text "__D4__" color: #64748b
        "__K5__": text "__D5__" color: #64748b
        _: text "Select a tool" color: #94a3b8
      }
    }
  }
}"""

for n, cfg in [
    ("editor", {"TITLE": "Text Editor", "DESC": "Text editor toolbar with formatting actions", "DEFAULT": "bold", "BAR_BG": "#f8fafc", "BTN_TEXT": "#ffffff", "IC1": "#1e293b", "B1": "B", "K1": "bold", "D1": "Bold text formatting active.", "IC2": "#1e293b", "B2": "I", "K2": "italic", "D2": "Italic text formatting active.", "IC3": "#1e293b", "B3": "U", "K3": "underline", "D3": "Underline formatting active.", "IC4": "#1e293b", "B4": "S", "K4": "strike", "D4": "Strikethrough formatting active.", "IC5": "#ef4444", "B5": "X", "K5": "clear", "D5": "Clear all formatting."}),
    ("drawing", {"TITLE": "Drawing Canvas", "DESC": "Drawing tool toolbar with brush selection", "DEFAULT": "pencil", "BAR_BG": "#1e293b", "BTN_TEXT": "#ffffff", "IC1": "#3b82f6", "B1": "/", "K1": "pencil", "D1": "Freehand pencil drawing tool.", "IC2": "#ef4444", "B2": "O", "K2": "circle", "D2": "Draw circles and ellipses.", "IC3": "#16a34a", "B3": "R", "K3": "rect-tool", "D3": "Draw rectangles and squares.", "IC4": "#f59e0b", "B4": "L", "K4": "line", "D4": "Draw straight lines.", "IC5": "#94a3b8", "B5": "E", "K5": "eraser", "D5": "Erase drawn elements."}),
    ("media", {"TITLE": "Media Player", "DESC": "Media player toolbar with playback controls", "DEFAULT": "play", "BAR_BG": "#0f172a", "BTN_TEXT": "#ffffff", "IC1": "#64748b", "B1": "<", "K1": "prev", "D1": "Skip to previous track.", "IC2": "#3b82f6", "B2": ">", "K2": "play", "D2": "Play the current track.", "IC3": "#64748b", "B3": "||", "K3": "pause", "D3": "Pause playback.", "IC4": "#64748b", "B4": ">", "K4": "next", "D4": "Skip to next track.", "IC5": "#ef4444", "B5": "x", "K5": "stop", "D5": "Stop playback completely."}),
    ("file-ops", {"TITLE": "File Manager Bar", "DESC": "File operations toolbar with CRUD actions", "DEFAULT": "new", "BAR_BG": "#f1f5f9", "BTN_TEXT": "#ffffff", "IC1": "#2563eb", "B1": "+", "K1": "new", "D1": "Create a new file.", "IC2": "#16a34a", "B2": "O", "K2": "open", "D2": "Open an existing file.", "IC3": "#7c3aed", "B3": "S", "K3": "save", "D3": "Save the current file.", "IC4": "#f59e0b", "B4": "C", "K4": "copy", "D4": "Copy the selected file.", "IC5": "#ef4444", "B5": "D", "K5": "delete", "D5": "Delete the selected file."}),
    ("mail-ops", {"TITLE": "Email Toolbar", "DESC": "Email action toolbar with message operations", "DEFAULT": "compose", "BAR_BG": "#ffffff", "BTN_TEXT": "#ffffff", "IC1": "#2563eb", "B1": "+", "K1": "compose", "D1": "Compose a new email.", "IC2": "#16a34a", "B2": "R", "K2": "reply", "D2": "Reply to the current email.", "IC3": "#0891b2", "B3": "F", "K3": "forward", "D3": "Forward this email.", "IC4": "#f59e0b", "B4": "*", "K4": "star", "D4": "Star this email.", "IC5": "#ef4444", "B5": "X", "K5": "trash", "D5": "Move email to trash."}),
    ("spreadsheet", {"TITLE": "Spreadsheet Bar", "DESC": "Spreadsheet toolbar with cell operations", "DEFAULT": "select", "BAR_BG": "#f0fdf4", "BTN_TEXT": "#ffffff", "IC1": "#16a34a", "B1": "^", "K1": "select", "D1": "Select cells for editing.", "IC2": "#2563eb", "B2": "=", "K2": "formula", "D2": "Insert a formula.", "IC3": "#7c3aed", "B3": "#", "K3": "format", "D3": "Format cell values.", "IC4": "#f59e0b", "B4": "v", "K4": "sort", "D4": "Sort selected column.", "IC5": "#ef4444", "B5": "X", "K5": "clear-cell", "D5": "Clear cell contents."}),
    ("photo", {"TITLE": "Photo Editor", "DESC": "Photo editing toolbar with adjustment tools", "DEFAULT": "crop", "BAR_BG": "#18181b", "BTN_TEXT": "#ffffff", "IC1": "#a78bfa", "B1": "C", "K1": "crop", "D1": "Crop the image to a region.", "IC2": "#38bdf8", "B2": "R", "K2": "rotate", "D2": "Rotate the image 90 degrees.", "IC3": "#fbbf24", "B3": "*", "K3": "brightness", "D3": "Adjust image brightness.", "IC4": "#f87171", "B4": "~", "K4": "filter", "D4": "Apply a photo filter.", "IC5": "#4ade80", "B5": "S", "K5": "save-photo", "D5": "Save edited photo."}),
    ("map", {"TITLE": "Map Controls", "DESC": "Map navigation toolbar with view controls", "DEFAULT": "pan", "BAR_BG": "#f8fafc", "BTN_TEXT": "#ffffff", "IC1": "#3b82f6", "B1": "+", "K1": "zoom-in", "D1": "Zoom in on the map.", "IC2": "#3b82f6", "B2": "-", "K2": "zoom-out", "D2": "Zoom out on the map.", "IC3": "#16a34a", "B3": "M", "K3": "pan", "D3": "Pan across the map.", "IC4": "#f59e0b", "B4": "P", "K4": "pin", "D4": "Drop a pin on the map.", "IC5": "#8b5cf6", "B5": "L", "K5": "layers", "D5": "Toggle map layer display."}),
    ("kanban", {"TITLE": "Kanban Toolbar", "DESC": "Kanban board toolbar with task actions", "DEFAULT": "add-task", "BAR_BG": "#f5f3ff", "BTN_TEXT": "#ffffff", "IC1": "#7c3aed", "B1": "+", "K1": "add-task", "D1": "Add a new task card.", "IC2": "#2563eb", "B2": "=", "K2": "filter-tasks", "D2": "Filter visible tasks.", "IC3": "#16a34a", "B3": "M", "K3": "move", "D3": "Move task between columns.", "IC4": "#f59e0b", "B4": "!", "K4": "priority", "D4": "Set task priority.", "IC5": "#ef4444", "B5": "X", "K5": "archive-task", "D5": "Archive the selected task."}),
    ("slides", {"TITLE": "Slide Toolbar", "DESC": "Presentation slide toolbar with controls", "DEFAULT": "add-slide", "BAR_BG": "#fef2f2", "BTN_TEXT": "#ffffff", "IC1": "#ef4444", "B1": "+", "K1": "add-slide", "D1": "Add a new slide.", "IC2": "#2563eb", "B2": "T", "K2": "add-text", "D2": "Insert a text box.", "IC3": "#16a34a", "B3": "I", "K3": "add-image", "D3": "Insert an image.", "IC4": "#7c3aed", "B4": ">", "K4": "present", "D4": "Start the presentation.", "IC5": "#64748b", "B5": "?", "K5": "help", "D5": "View keyboard shortcuts."}),
]:
    ex(f"gen-toolbar-{n}.naze", cfg["DESC"], fill(TOOLBAR_T, cfg))


# ═══════════════════════════════════════════════════════════════════════════════
# BATCH D: 100 examples across 10 new categories
# Categories: review, schedule, recipe, news, score, playlist,
#             address, compare, countdown, filter
# ═══════════════════════════════════════════════════════════════════════════════


# ─── Generator: Product Reviews (gen-review-*) ───────────────────────────────

REVIEW_CARD_T = """-- __DESC__
app "__TITLE__" {
  state reviews = [__ITEMS__]
  state rating-filter = "__DEFAULT_FILTER__"
  computed total-reviews = reviews | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{total-reviews} reviews" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 36px, color: __CLR__, radius: 4px {
        text "All" color: #ffffff
        on click: set rating-filter = "all"
      }
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "__FILTER_LABEL__" color: #ffffff
        on click: set rating-filter = "__FILTER_VAL__"
      }
    }

    each review in reviews {
      rect padding: 12px, color: __BG__, radius: 8px {
        text "{review.__F1__}" font-weight: bold
        text "{review.__F2__}" color: #f59e0b
        text "{review.__F3__}" color: #64748b
      }
    }
  }
}"""

for n, cfg in [
    ("products", {"TITLE": "Product Reviews", "DESC": "Product review cards with star ratings",
                  "ITEMS": '{author: "Alice", stars: "5", comment: "Excellent quality!"}, {author: "Bob", stars: "4", comment: "Good value for money"}, {author: "Carol", stars: "3", comment: "Average product"}',
                  "DEFAULT_FILTER": "all", "FILTER_LABEL": "5 Stars", "FILTER_VAL": "5",
                  "F1": "author", "F2": "stars", "F3": "comment", "BG": "#fefce8", "CLR": "#2563eb"}),
    ("restaurants", {"TITLE": "Restaurant Reviews", "DESC": "Restaurant review cards with food ratings",
                     "ITEMS": '{author: "Dave", stars: "5", comment: "Amazing pasta!"}, {author: "Eve", stars: "4", comment: "Great ambiance"}, {author: "Frank", stars: "5", comment: "Best pizza in town"}',
                     "DEFAULT_FILTER": "all", "FILTER_LABEL": "Top", "FILTER_VAL": "5",
                     "F1": "author", "F2": "stars", "F3": "comment", "BG": "#fff7ed", "CLR": "#ea580c"}),
    ("books", {"TITLE": "Book Reviews", "DESC": "Book review cards with reader ratings",
               "ITEMS": '{author: "Reader1", stars: "5", comment: "A masterpiece"}, {author: "Reader2", stars: "3", comment: "Decent read"}, {author: "Reader3", stars: "4", comment: "Engaging plot"}',
               "DEFAULT_FILTER": "all", "FILTER_LABEL": "4+ Stars", "FILTER_VAL": "4",
               "F1": "author", "F2": "stars", "F3": "comment", "BG": "#f0fdf4", "CLR": "#16a34a"}),
    ("movies", {"TITLE": "Movie Reviews", "DESC": "Movie review cards with critic scores",
                "ITEMS": '{author: "Critic1", stars: "4", comment: "Visually stunning"}, {author: "Critic2", stars: "5", comment: "Oscar-worthy"}, {author: "Critic3", stars: "3", comment: "Entertaining enough"}',
                "DEFAULT_FILTER": "all", "FILTER_LABEL": "5 Stars", "FILTER_VAL": "5",
                "F1": "author", "F2": "stars", "F3": "comment", "BG": "#fdf2f8", "CLR": "#db2777"}),
    ("hotels", {"TITLE": "Hotel Reviews", "DESC": "Hotel review cards with guest feedback",
                "ITEMS": '{author: "Guest1", stars: "5", comment: "Spotless rooms"}, {author: "Guest2", stars: "4", comment: "Great location"}, {author: "Guest3", stars: "2", comment: "Noisy at night"}',
                "DEFAULT_FILTER": "all", "FILTER_LABEL": "Top", "FILTER_VAL": "5",
                "F1": "author", "F2": "stars", "F3": "comment", "BG": "#eff6ff", "CLR": "#2563eb"}),
]:
    ex(f"gen-review-{n}.naze", cfg["DESC"], fill(REVIEW_CARD_T, cfg))

# --- Hand-crafted reviews for variety ---

ex("gen-review-summary.naze", "Review summary with average rating and breakdown",
   """-- Review summary dashboard
app "Review Summary" {
  state total = 124
  state avg-rating = 4
  state five-star = 68
  state four-star = 32
  state three-star = 14
  state two-star = 7
  state one-star = 3

  column padding: 20px, gap: 16px {
    heading "Review Summary"
    text "Average: {avg-rating} / 5" font-size: 28px, color: #f59e0b
    text "{total} total reviews" color: #64748b

    column gap: 8px {
      row gap: 8px {
        text "5 stars" font-weight: bold
        rect width: 200px, height: 16px, color: #fde68a, radius: 4px
        text "{five-star}" color: #64748b
      }
      row gap: 8px {
        text "4 stars" font-weight: bold
        rect width: 150px, height: 16px, color: #fde68a, radius: 4px
        text "{four-star}" color: #64748b
      }
      row gap: 8px {
        text "3 stars" font-weight: bold
        rect width: 80px, height: 16px, color: #fde68a, radius: 4px
        text "{three-star}" color: #64748b
      }
      row gap: 8px {
        text "2 stars" font-weight: bold
        rect width: 40px, height: 16px, color: #fde68a, radius: 4px
        text "{two-star}" color: #64748b
      }
      row gap: 8px {
        text "1 star" font-weight: bold
        rect width: 20px, height: 16px, color: #fde68a, radius: 4px
        text "{one-star}" color: #64748b
      }
    }
  }
}""")

ex("gen-review-form.naze", "Review submission form with rating select",
   """-- Review submission form
app "Write a Review" {
  state reviewer = ""
  state rating = "5"
  state comment = ""
  state submitted = false

  column padding: 20px, gap: 16px {
    heading "Write a Review"

    input bind: reviewer, placeholder: "Your name"

    select bind: rating {
      option "5 Stars" value: "5"
      option "4 Stars" value: "4"
      option "3 Stars" value: "3"
      option "2 Stars" value: "2"
      option "1 Star" value: "1"
    }

    input bind: comment, placeholder: "Write your review..."

    rect width: 140px, height: 40px, color: #f59e0b, radius: 8px {
      text "Submit Review" color: #ffffff
      on click: set submitted = true
    }

    if submitted {
      text "Thank you for your review!" color: #16a34a
    }
  }
}""")

ex("gen-review-featured.naze", "Featured review highlight card",
   """-- Featured review
app "Featured Review" {
  state featured-author = "Sarah M."
  state featured-rating = 5
  state featured-text = "This product changed my workflow completely. Highly recommend!"

  column padding: 20px, gap: 16px {
    heading "Featured Review"

    rect padding: 20px, color: #fefce8, radius: 12px {
      row gap: 8px {
        rect width: 48px, height: 48px, color: #f59e0b, radius: 24px {
          text "S" color: #ffffff, font-size: 20px
        }
        column gap: 4px {
          text "{featured-author}" font-weight: bold, font-size: 18px
          text "{featured-rating} / 5 stars" color: #f59e0b
        }
      }
      text "{featured-text}" color: #374151, font-size: 16px
    }

    text "Was this review helpful?" color: #64748b
    row gap: 8px {
      rect width: 60px, height: 32px, color: #e2e8f0, radius: 4px {
        text "Yes"
      }
      rect width: 60px, height: 32px, color: #e2e8f0, radius: 4px {
        text "No"
      }
    }
  }
}""")

ex("gen-review-comparison.naze", "Side-by-side review comparison for two products",
   """-- Product review comparison
app "Review Comparison" {
  state product-a-rating = 4
  state product-b-rating = 3
  state product-a-count = 89
  state product-b-count = 54

  column padding: 20px, gap: 16px {
    heading "Review Comparison"

    grid columns: 2, gap: 16px {
      rect padding: 16px, color: #eff6ff, radius: 8px {
        text "Widget Pro" font-weight: bold, font-size: 18px
        text "{product-a-rating} / 5" color: #f59e0b, font-size: 24px
        text "{product-a-count} reviews" color: #64748b
        text "Pros: Durable, fast" color: #16a34a
        text "Cons: Pricey" color: #dc2626
      }
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "Widget Lite" font-weight: bold, font-size: 18px
        text "{product-b-rating} / 5" color: #f59e0b, font-size: 24px
        text "{product-b-count} reviews" color: #64748b
        text "Pros: Affordable" color: #16a34a
        text "Cons: Slower" color: #dc2626
      }
    }
  }
}""")

ex("gen-review-testimonials.naze", "Testimonial cards with customer quotes",
   """-- Testimonials
app "Testimonials" {
  state testimonials = [{name: "James K.", quote: "Saved us hours every week"}, {name: "Maria L.", quote: "Best investment this year"}, {name: "Chen W.", quote: "Support team is incredible"}]

  column padding: 20px, gap: 16px {
    heading "What Customers Say"

    each t in testimonials {
      rect padding: 16px, color: #f8fafc, radius: 12px {
        text "{t.quote}" font-size: 16px, color: #374151
        separator
        text "-- {t.name}" color: #64748b
      }
    }
  }
}""")


# ─── Generator: Schedules / Timetables (gen-schedule-*) ──────────────────────

SCHEDULE_T = """-- __DESC__
app "__TITLE__" {
  state events = [__ITEMS__]
  computed event-count = events | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{event-count} __LABEL__" color: #64748b

    each evt in events | sort-by __SORT__ {
      row padding: 12px, color: __BG__, radius: 8px, gap: 12px {
        rect width: 60px, height: 40px, color: __CLR__, radius: 4px {
          text "{evt.__TIME_FIELD__}" color: #ffffff, font-size: 12px
        }
        column gap: 2px {
          text "{evt.__F1__}" font-weight: bold
          text "{evt.__F2__}" color: #64748b
        }
      }
    }
  }
}"""

for n, cfg in [
    ("weekly", {"TITLE": "Weekly Schedule", "DESC": "Weekly schedule with day and time slots",
                "ITEMS": '{day: "Monday", time: "09:00", activity: "Team Standup", room: "Room A"}, {day: "Tuesday", time: "14:00", activity: "Design Review", room: "Room B"}, {day: "Wednesday", time: "10:00", activity: "Sprint Planning", room: "Room C"}',
                "LABEL": "events this week", "SORT": "day", "TIME_FIELD": "time", "F1": "activity", "F2": "room", "BG": "#eff6ff", "CLR": "#2563eb"}),
    ("class", {"TITLE": "Class Timetable", "DESC": "School class timetable with subjects",
               "ITEMS": '{period: "1", time: "08:30", subject: "Mathematics", teacher: "Dr. Smith"}, {period: "2", time: "09:30", subject: "Physics", teacher: "Prof. Lee"}, {period: "3", time: "10:30", subject: "English", teacher: "Ms. Brown"}, {period: "4", time: "11:30", subject: "History", teacher: "Mr. Davis"}',
               "LABEL": "classes today", "SORT": "period", "TIME_FIELD": "time", "F1": "subject", "F2": "teacher", "BG": "#fefce8", "CLR": "#ca8a04"}),
    ("gym", {"TITLE": "Gym Schedule", "DESC": "Gym workout schedule with time slots",
             "ITEMS": '{day: "Mon", time: "06:00", activity: "Chest and Triceps", trainer: "Coach Mike"}, {day: "Tue", time: "06:00", activity: "Back and Biceps", trainer: "Coach Sarah"}, {day: "Wed", time: "07:00", activity: "Legs", trainer: "Coach Mike"}',
             "LABEL": "workouts", "SORT": "day", "TIME_FIELD": "time", "F1": "activity", "F2": "trainer", "BG": "#ecfdf5", "CLR": "#16a34a"}),
    ("meetings", {"TITLE": "Meeting Schedule", "DESC": "Daily meeting schedule with participants",
                  "ITEMS": '{slot: "1", time: "09:00", meeting: "All Hands", attendees: "Full team"}, {slot: "2", time: "11:00", meeting: "1-on-1", attendees: "Manager"}, {slot: "3", time: "15:00", meeting: "Code Review", attendees: "Dev team"}',
                  "LABEL": "meetings today", "SORT": "slot", "TIME_FIELD": "time", "F1": "meeting", "F2": "attendees", "BG": "#f0f9ff", "CLR": "#0284c7"}),
    ("shifts", {"TITLE": "Shift Roster", "DESC": "Employee shift schedule with roles",
                "ITEMS": '{order: "1", time: "06:00", employee: "Alice - Morning", role: "Front Desk"}, {order: "2", time: "14:00", employee: "Bob - Afternoon", role: "Support"}, {order: "3", time: "22:00", employee: "Carol - Night", role: "Security"}',
                "LABEL": "shifts today", "SORT": "order", "TIME_FIELD": "time", "F1": "employee", "F2": "role", "BG": "#faf5ff", "CLR": "#7c3aed"}),
]:
    ex(f"gen-schedule-{n}.naze", cfg["DESC"], fill(SCHEDULE_T, cfg))

ex("gen-schedule-calendar.naze", "Monthly calendar view with event markers",
   """-- Monthly calendar
app "Calendar Events" {
  state month = "February"
  state year = 2026
  state events = [{date: "Feb 3", title: "Dentist"}, {date: "Feb 14", title: "Valentine's Day"}, {date: "Feb 20", title: "Team Retreat"}]
  computed event-count = events | count

  column padding: 20px, gap: 16px {
    heading "{month} {year}"
    text "{event-count} events this month" color: #64748b

    each evt in events {
      row padding: 10px, color: #eff6ff, radius: 8px, gap: 8px {
        text "{evt.date}" font-weight: bold, color: #2563eb
        text "{evt.title}"
      }
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Prev"
      }
      rect width: 80px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Next"
      }
    }
  }
}""")

ex("gen-schedule-agenda.naze", "Today's agenda with time blocks and priorities",
   """-- Daily agenda
app "Today's Agenda" {
  state tasks = [{time: "08:00", task: "Morning standup", priority: "high"}, {time: "10:00", task: "Deep work block", priority: "high"}, {time: "12:00", task: "Lunch break", priority: "low"}, {time: "14:00", task: "Client call", priority: "high"}, {time: "16:00", task: "Code review", priority: "medium"}]

  column padding: 20px, gap: 12px {
    heading "Today's Agenda"

    each item in tasks {
      row padding: 10px, color: #f8fafc, radius: 8px, gap: 12px {
        text "{item.time}" font-weight: bold, color: #2563eb
        text "{item.task}" font-size: 16px
        match item.priority {
          "high": text "HIGH" color: #dc2626, font-size: 12px
          "medium": text "MED" color: #f59e0b, font-size: 12px
          _: text "LOW" color: #16a34a, font-size: 12px
        }
      }
    }
  }
}""")

ex("gen-schedule-booking.naze", "Appointment booking with time slot selection",
   """-- Appointment booking
app "Book Appointment" {
  state selected-slot = "none"
  state booked = false

  column padding: 20px, gap: 16px {
    heading "Book Appointment"
    text "Select a time slot" color: #64748b

    column gap: 8px {
      rect width: 200px, height: 40px, color: #ecfdf5, radius: 8px {
        text "09:00 - Available"
        on click: set selected-slot = "09:00"
      }
      rect width: 200px, height: 40px, color: #ecfdf5, radius: 8px {
        text "11:00 - Available"
        on click: set selected-slot = "11:00"
      }
      rect width: 200px, height: 40px, color: #fce7f3, radius: 8px {
        text "13:00 - Taken" color: #9ca3af
      }
      rect width: 200px, height: 40px, color: #ecfdf5, radius: 8px {
        text "15:00 - Available"
        on click: set selected-slot = "15:00"
      }
    }

    if selected-slot != "none" {
      text "Selected: {selected-slot}" font-weight: bold, color: #2563eb
      rect width: 140px, height: 40px, color: #2563eb, radius: 8px {
        text "Confirm" color: #ffffff
        on click: set booked = true
      }
    }

    if booked {
      text "Appointment booked!" color: #16a34a, font-size: 18px
    }
  }
}""")

ex("gen-schedule-countdown-event.naze", "Event schedule with days until each event",
   """-- Event countdown schedule
app "Upcoming Events" {
  state events = [{name: "Conference", days: "12", location: "NYC"}, {name: "Workshop", days: "25", location: "London"}, {name: "Hackathon", days: "40", location: "Berlin"}]

  column padding: 20px, gap: 16px {
    heading "Upcoming Events"

    each evt in events | sort-by days {
      rect padding: 16px, color: #f8fafc, radius: 8px {
        row gap: 12px {
          rect width: 60px, height: 60px, color: #6366f1, radius: 8px {
            text "{evt.days}" color: #ffffff, font-size: 24px
          }
          column gap: 4px {
            text "{evt.name}" font-weight: bold, font-size: 18px
            text "{evt.location}" color: #64748b
            text "{evt.days} days away" color: #6366f1
          }
        }
      }
    }
  }
}""")

ex("gen-schedule-weekly-planner.naze", "Weekly planner with day columns and tasks",
   """-- Weekly planner
app "Weekly Planner" {
  state selected-day = "Monday"

  column padding: 20px, gap: 16px {
    heading "Weekly Planner"

    row gap: 4px {
      rect width: 50px, height: 32px, color: #2563eb, radius: 4px {
        text "Mon" color: #ffffff, font-size: 12px
        on click: set selected-day = "Monday"
      }
      rect width: 50px, height: 32px, color: #64748b, radius: 4px {
        text "Tue" color: #ffffff, font-size: 12px
        on click: set selected-day = "Tuesday"
      }
      rect width: 50px, height: 32px, color: #64748b, radius: 4px {
        text "Wed" color: #ffffff, font-size: 12px
        on click: set selected-day = "Wednesday"
      }
      rect width: 50px, height: 32px, color: #64748b, radius: 4px {
        text "Thu" color: #ffffff, font-size: 12px
        on click: set selected-day = "Thursday"
      }
      rect width: 50px, height: 32px, color: #64748b, radius: 4px {
        text "Fri" color: #ffffff, font-size: 12px
        on click: set selected-day = "Friday"
      }
    }

    text "Showing: {selected-day}" font-weight: bold, color: #2563eb

    match selected-day {
      "Monday": text "Standup, Sprint Review, Deep Work"
      "Tuesday": text "Design sync, Code reviews"
      "Wednesday": text "Client demo, Planning"
      "Thursday": text "Pair programming, Retro"
      _: text "Deploy prep, Team lunch"
    }
  }
}""")


# ─── Generator: Recipe Cards (gen-recipe-*) ──────────────────────────────────

RECIPE_T = """-- __DESC__
app "__TITLE__" {
  state servings = __SERVINGS__
  state ingredients = [__INGREDIENTS__]
  state current-step = 1

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__CUISINE__ | __TIME__ | {servings} servings" color: #64748b

    rect padding: 16px, color: __BG__, radius: 8px {
      text "Ingredients" font-weight: bold, font-size: 18px
      each item in ingredients {
        text "- {item.name}: {item.amount}" color: #374151
      }
    }

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "Step {current-step}" font-weight: bold, font-size: 18px, color: __CLR__
      text "__STEP_TEXT__" color: #374151
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: __CLR__, radius: 4px {
        text "Next Step" color: #ffffff
        on click: set current-step = current-step + 1
      }
      rect width: 100px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Reset"
        on click: set current-step = 1
      }
    }
  }
}"""

for n, cfg in [
    ("pasta", {"TITLE": "Spaghetti Carbonara", "DESC": "Classic carbonara recipe card",
               "SERVINGS": "4", "CUISINE": "Italian", "TIME": "25 min",
               "INGREDIENTS": '{name: "Spaghetti", amount: "400g"}, {name: "Guanciale", amount: "200g"}, {name: "Eggs", amount: "4"}, {name: "Pecorino", amount: "100g"}',
               "STEP_TEXT": "Boil pasta, fry guanciale, mix eggs with cheese, combine", "BG": "#fff7ed", "CLR": "#ea580c"}),
    ("curry", {"TITLE": "Thai Green Curry", "DESC": "Thai green curry recipe with coconut milk",
               "SERVINGS": "4", "CUISINE": "Thai", "TIME": "35 min",
               "INGREDIENTS": '{name: "Chicken", amount: "500g"}, {name: "Coconut Milk", amount: "400ml"}, {name: "Green Paste", amount: "3 tbsp"}, {name: "Basil", amount: "1 cup"}',
               "STEP_TEXT": "Cook paste, add chicken, pour coconut milk, simmer with basil", "BG": "#ecfdf5", "CLR": "#16a34a"}),
    ("tacos", {"TITLE": "Fish Tacos", "DESC": "Baja-style fish taco recipe",
               "SERVINGS": "6", "CUISINE": "Mexican", "TIME": "30 min",
               "INGREDIENTS": '{name: "White Fish", amount: "500g"}, {name: "Tortillas", amount: "12"}, {name: "Cabbage", amount: "2 cups"}, {name: "Lime", amount: "3"}',
               "STEP_TEXT": "Season fish, grill, shred cabbage, assemble tacos", "BG": "#fefce8", "CLR": "#ca8a04"}),
    ("salad", {"TITLE": "Caesar Salad", "DESC": "Classic Caesar salad recipe card",
               "SERVINGS": "2", "CUISINE": "American", "TIME": "15 min",
               "INGREDIENTS": '{name: "Romaine", amount: "2 heads"}, {name: "Croutons", amount: "1 cup"}, {name: "Parmesan", amount: "50g"}, {name: "Dressing", amount: "4 tbsp"}',
               "STEP_TEXT": "Wash lettuce, make dressing, toss with croutons and cheese", "BG": "#f0fdf4", "CLR": "#22c55e"}),
    ("sushi", {"TITLE": "Salmon Sushi Roll", "DESC": "Homemade sushi roll recipe",
               "SERVINGS": "4", "CUISINE": "Japanese", "TIME": "45 min",
               "INGREDIENTS": '{name: "Sushi Rice", amount: "2 cups"}, {name: "Salmon", amount: "300g"}, {name: "Nori", amount: "5 sheets"}, {name: "Avocado", amount: "1"}',
               "STEP_TEXT": "Cook rice, slice fish, roll with nori, cut into pieces", "BG": "#fce7f3", "CLR": "#db2777"}),
]:
    ex(f"gen-recipe-{n}.naze", cfg["DESC"], fill(RECIPE_T, cfg))

ex("gen-recipe-nutrition.naze", "Recipe with nutrition facts panel",
   """-- Recipe with nutrition
app "Healthy Bowl" {
  state calories = 450
  state protein = 32
  state carbs = 48
  state fat = 14

  column padding: 20px, gap: 16px {
    heading "Healthy Bowl"
    text "Grain bowl with chicken and veggies" color: #64748b

    rect padding: 16px, color: #f0fdf4, radius: 8px {
      text "Nutrition Facts" font-weight: bold, font-size: 18px
      separator
      row gap: 16px {
        column gap: 4px {
          text "{calories}" font-size: 24px, color: #16a34a
          text "Calories" color: #64748b, font-size: 12px
        }
        column gap: 4px {
          text "{protein}g" font-size: 24px, color: #2563eb
          text "Protein" color: #64748b, font-size: 12px
        }
        column gap: 4px {
          text "{carbs}g" font-size: 24px, color: #f59e0b
          text "Carbs" color: #64748b, font-size: 12px
        }
        column gap: 4px {
          text "{fat}g" font-size: 24px, color: #ef4444
          text "Fat" color: #64748b, font-size: 12px
        }
      }
    }
  }
}""")

ex("gen-recipe-timer.naze", "Recipe with built-in cooking timer",
   """-- Recipe with timer
app "Slow Roast Chicken" {
  state cook-time = 90
  state stage = "prep"

  timer oven: every 1s {
    set cook-time = cook-time - 1
  }

  column padding: 20px, gap: 16px {
    heading "Slow Roast Chicken"
    text "Roast at 350F" color: #64748b

    rect padding: 16px, color: #fef3c7, radius: 8px {
      text "{cook-time} minutes remaining" font-size: 28px, color: #b45309
    }

    match stage {
      "prep": text "Season and truss the chicken" font-size: 16px
      "cooking": text "In the oven - do not open!" font-size: 16px, color: #ef4444
      _: text "Let rest 15 minutes before carving" font-size: 16px
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "Prep" color: #ffffff
        on click: set stage = "prep"
      }
      rect width: 80px, height: 36px, color: #ef4444, radius: 4px {
        text "Cook" color: #ffffff
        on click: set stage = "cooking"
      }
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Done" color: #ffffff
        on click: set stage = "done"
      }
    }
  }
}""")

ex("gen-recipe-collection.naze", "Recipe collection browser with category filter",
   """-- Recipe collection
app "My Recipes" {
  state category = "all"
  state recipes = [{name: "Pasta Primavera", cat: "Italian", time: "20 min"}, {name: "Pad Thai", cat: "Asian", time: "25 min"}, {name: "Beef Stew", cat: "Comfort", time: "90 min"}, {name: "Greek Salad", cat: "Mediterranean", time: "10 min"}]
  computed recipe-count = recipes | count

  column padding: 20px, gap: 16px {
    heading "My Recipes"
    text "{recipe-count} recipes" color: #64748b

    row gap: 8px {
      rect width: 60px, height: 32px, color: #2563eb, radius: 4px {
        text "All" color: #ffffff, font-size: 12px
        on click: set category = "all"
      }
      rect width: 70px, height: 32px, color: #ea580c, radius: 4px {
        text "Italian" color: #ffffff, font-size: 12px
        on click: set category = "Italian"
      }
      rect width: 60px, height: 32px, color: #16a34a, radius: 4px {
        text "Asian" color: #ffffff, font-size: 12px
        on click: set category = "Asian"
      }
    }

    each r in recipes {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 8px {
        text "{r.name}" font-weight: bold
        text "{r.cat}" color: #6366f1, font-size: 12px
        text "{r.time}" color: #64748b
      }
    }
  }
}""")

ex("gen-recipe-steps.naze", "Step-by-step recipe with progress indicator",
   """-- Step by step recipe
app "Banana Bread" {
  state step = 1
  state total-steps = 5

  column padding: 20px, gap: 16px {
    heading "Banana Bread"
    text "Step {step} of {total-steps}" color: #64748b

    rect width: 300px, height: 8px, color: #e2e8f0, radius: 4px

    match step {
      1: text "Preheat oven to 350F. Grease a loaf pan."
      2: text "Mash 3 ripe bananas in a large bowl."
      3: text "Mix in melted butter, sugar, egg, and vanilla."
      4: text "Fold in flour, baking soda, and salt."
      _: text "Pour into pan and bake 55-60 minutes."
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "Previous" color: #ffffff
        on click: set step = step - 1
      }
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "Next" color: #ffffff
        on click: set step = step + 1
      }
    }
  }
}""")

ex("gen-recipe-shopping.naze", "Shopping list generated from recipe ingredients",
   """-- Recipe shopping list
app "Shopping List" {
  state items = [{ingredient: "Flour", qty: "2 cups", checked: "no"}, {ingredient: "Sugar", qty: "1 cup", checked: "no"}, {ingredient: "Butter", qty: "100g", checked: "no"}, {ingredient: "Eggs", qty: "3", checked: "no"}, {ingredient: "Milk", qty: "250ml", checked: "no"}]
  computed total-items = items | count

  column padding: 20px, gap: 12px {
    heading "Shopping List"
    text "{total-items} items needed" color: #64748b

    each item in items {
      row padding: 10px, color: #f8fafc, radius: 8px, gap: 8px {
        checkbox bind: item.checked, label: "{item.ingredient}"
        text "{item.qty}" color: #64748b
      }
    }
  }
}""")


# ─── Generator: News Articles (gen-news-*) ───────────────────────────────────

NEWS_T = """-- __DESC__
app "__TITLE__" {
  state articles = [__ITEMS__]
  state section = "__DEFAULT_SECTION__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"

    row gap: 8px {
      rect width: 80px, height: 32px, color: __CLR__, radius: 4px {
        text "__S1__" color: #ffffff, font-size: 12px
        on click: set section = "__S1_VAL__"
      }
      rect width: 80px, height: 32px, color: #64748b, radius: 4px {
        text "__S2__" color: #ffffff, font-size: 12px
        on click: set section = "__S2_VAL__"
      }
    }

    each article in articles {
      rect padding: 12px, color: __BG__, radius: 8px {
        text "{article.__F1__}" font-weight: bold, font-size: 16px
        text "{article.__F2__}" color: #64748b
        text "{article.__F3__}" color: __CLR__, font-size: 12px
      }
    }
  }
}"""

for n, cfg in [
    ("tech", {"TITLE": "Tech News", "DESC": "Technology news feed with section filters",
              "ITEMS": '{headline: "AI Model Breaks Records", source: "TechCrunch", time: "2h ago"}, {headline: "New Chip Architecture Unveiled", source: "Wired", time: "4h ago"}, {headline: "Open Source Framework Hits 1M Stars", source: "GitHub Blog", time: "6h ago"}',
              "DEFAULT_SECTION": "latest", "S1": "Latest", "S1_VAL": "latest", "S2": "Trending", "S2_VAL": "trending",
              "F1": "headline", "F2": "source", "F3": "time", "BG": "#eff6ff", "CLR": "#2563eb"}),
    ("sports", {"TITLE": "Sports News", "DESC": "Sports news with live updates",
                "ITEMS": '{headline: "Championship Final Set", source: "ESPN", time: "1h ago"}, {headline: "Record Transfer Fee Agreed", source: "Sky Sports", time: "3h ago"}, {headline: "Olympic Qualification Round Results", source: "NBC Sports", time: "5h ago"}',
                "DEFAULT_SECTION": "live", "S1": "Live", "S1_VAL": "live", "S2": "Scores", "S2_VAL": "scores",
                "F1": "headline", "F2": "source", "F3": "time", "BG": "#ecfdf5", "CLR": "#16a34a"}),
    ("business", {"TITLE": "Business News", "DESC": "Business and finance news ticker",
                  "ITEMS": '{headline: "Markets Rally on Rate Decision", source: "Bloomberg", time: "30m ago"}, {headline: "Startup Raises $500M Series D", source: "Reuters", time: "2h ago"}, {headline: "Quarterly Earnings Beat Estimates", source: "CNBC", time: "4h ago"}',
                  "DEFAULT_SECTION": "markets", "S1": "Markets", "S1_VAL": "markets", "S2": "Deals", "S2_VAL": "deals",
                  "F1": "headline", "F2": "source", "F3": "time", "BG": "#fefce8", "CLR": "#ca8a04"}),
    ("world", {"TITLE": "World News", "DESC": "International news headlines",
               "ITEMS": '{headline: "Climate Summit Reaches Agreement", source: "BBC", time: "1h ago"}, {headline: "Space Mission Launches Successfully", source: "AP News", time: "3h ago"}, {headline: "Historic Trade Deal Signed", source: "Al Jazeera", time: "5h ago"}',
               "DEFAULT_SECTION": "top", "S1": "Top", "S1_VAL": "top", "S2": "Region", "S2_VAL": "region",
               "F1": "headline", "F2": "source", "F3": "time", "BG": "#f0f9ff", "CLR": "#0284c7"}),
    ("science", {"TITLE": "Science Daily", "DESC": "Science and research news feed",
                 "ITEMS": '{headline: "New Exoplanet Discovered in Habitable Zone", source: "Nature", time: "2h ago"}, {headline: "Gene Therapy Trial Shows Promise", source: "Science", time: "4h ago"}, {headline: "Quantum Computing Milestone Reached", source: "MIT Review", time: "6h ago"}',
                 "DEFAULT_SECTION": "latest", "S1": "Latest", "S1_VAL": "latest", "S2": "Popular", "S2_VAL": "popular",
                 "F1": "headline", "F2": "source", "F3": "time", "BG": "#faf5ff", "CLR": "#7c3aed"}),
]:
    ex(f"gen-news-{n}.naze", cfg["DESC"], fill(NEWS_T, cfg))

ex("gen-news-banner.naze", "Breaking news banner with alert styling",
   """-- Breaking news banner
app "Breaking News" {
  state alert = "Major earthquake reported in Pacific region"
  state is-live = true

  column padding: 0px, gap: 0px {
    if is-live {
      rect width: 600px, height: 48px, color: #dc2626, radius: 0px {
        row gap: 8px, padding: 12px {
          text "BREAKING" color: #ffffff, font-weight: bold
          text "{alert}" color: #ffffff
        }
      }
    }

    column padding: 20px, gap: 16px {
      heading "News Channel"
      text "Stay tuned for updates" color: #64748b

      rect width: 120px, height: 36px, color: #64748b, radius: 4px {
        text "Dismiss" color: #ffffff
        on click: set is-live = false
      }
    }
  }
}""")

ex("gen-news-ticker.naze", "Scrolling news ticker with rotating headlines",
   """-- News ticker
app "News Ticker" {
  state headline-index = 0

  timer rotate: every 3s {
    set headline-index = headline-index + 1
  }

  column padding: 20px, gap: 16px {
    heading "Live News"

    rect width: 500px, height: 40px, color: #1e293b, radius: 4px {
      match headline-index {
        0: text "Markets close at record high" color: #fbbf24
        1: text "Weather alert for eastern coast" color: #fbbf24
        2: text "Tech conference keynote announced" color: #fbbf24
        _: text "Sports: Championship finals tonight" color: #fbbf24
      }
    }

    text "Headline #{headline-index}" color: #64748b, font-size: 12px
  }
}""")

ex("gen-news-reading-list.naze", "Saved articles reading list with bookmarks",
   """-- Reading list
app "Reading List" {
  state saved = [{title: "The Future of Computing", source: "MIT Review", saved: "true"}, {title: "Climate Action Report", source: "UN", saved: "true"}, {title: "Design Systems Guide", source: "Figma", saved: "true"}]
  computed saved-count = saved | count

  column padding: 20px, gap: 16px {
    heading "Reading List"
    text "{saved-count} saved articles" color: #64748b

    each article in saved {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 8px {
        rect width: 4px, height: 40px, color: #2563eb, radius: 2px
        column gap: 2px {
          text "{article.title}" font-weight: bold
          text "{article.source}" color: #64748b, font-size: 12px
        }
      }
    }
  }
}""")

ex("gen-news-category-page.naze", "News page with categories and article list",
   """-- News categories
app "News Hub" {
  column padding: 20px, gap: 16px {
    heading "News Hub"
    text "Select a category" color: #64748b
    link "Technology", to: "/tech"
    link "Sports", to: "/sports"
    link "Business", to: "/business"
  }
}

page "/tech" {
  column padding: 20px, gap: 12px {
    heading "Technology"
    text "Latest tech articles"
    text "AI advances reshape the industry" font-weight: bold
    text "New programming language gains traction"
    link "Back to Hub", to: "/"
  }
}

page "/sports" {
  column padding: 20px, gap: 12px {
    heading "Sports"
    text "Latest sports coverage"
    text "Championship series goes to game 7" font-weight: bold
    text "Olympic qualifiers continue"
    link "Back to Hub", to: "/"
  }
}

page "/business" {
  column padding: 20px, gap: 12px {
    heading "Business"
    text "Market and finance news"
    text "Central bank holds interest rates" font-weight: bold
    text "Startup funding reaches new heights"
    link "Back to Hub", to: "/"
  }
}""")

ex("gen-news-digest.naze", "Daily news digest with summary cards",
   """-- Daily digest
app "Daily Digest" {
  state date = "February 17, 2026"
  state stories = [{title: "Tech Giant Launches New Product", summary: "Revolutionary device unveiled at keynote event"}, {title: "Economic Growth Exceeds Forecasts", summary: "GDP growth reaches 3.2% in Q4"}, {title: "Major Scientific Discovery", summary: "Researchers identify New particle at CERN"}]

  column padding: 20px, gap: 16px {
    heading "Daily Digest"
    text "{date}" color: #64748b

    separator

    each story in stories {
      rect padding: 16px, color: #f8fafc, radius: 8px {
        text "{story.title}" font-weight: bold, font-size: 16px
        text "{story.summary}" color: #64748b
      }
    }
  }
}""")


# ─── Generator: Scoreboards (gen-score-*) ────────────────────────────────────

SCORE_T = """-- __DESC__
app "__TITLE__" {
  state __TEAM1_VAR__ = __SCORE1__
  state __TEAM2_VAR__ = __SCORE2__
  state period = "__PERIOD__"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{period}" color: #64748b

    grid columns: 2, gap: 16px {
      rect padding: 20px, color: __CLR1__, radius: 8px {
        text "__TEAM1__" color: #ffffff, font-weight: bold
        text "{__TEAM1_VAR__}" color: #ffffff, font-size: 48px
        rect width: 80px, height: 32px, color: #ffffff, radius: 4px {
          text "+__INC__"
          on click: set __TEAM1_VAR__ = __TEAM1_VAR__ + __INC__
        }
      }
      rect padding: 20px, color: __CLR2__, radius: 8px {
        text "__TEAM2__" color: #ffffff, font-weight: bold
        text "{__TEAM2_VAR__}" color: #ffffff, font-size: 48px
        rect width: 80px, height: 32px, color: #ffffff, radius: 4px {
          text "+__INC__"
          on click: set __TEAM2_VAR__ = __TEAM2_VAR__ + __INC__
        }
      }
    }
  }
}"""

for n, cfg in [
    ("basketball", {"TITLE": "Basketball Score", "DESC": "Basketball game scoreboard with team scores",
                    "TEAM1": "Lakers", "TEAM1_VAR": "lakers-score", "SCORE1": "0",
                    "TEAM2": "Celtics", "TEAM2_VAR": "celtics-score", "SCORE2": "0",
                    "PERIOD": "Q1", "INC": "2", "CLR1": "#7c3aed", "CLR2": "#16a34a"}),
    ("soccer", {"TITLE": "Soccer Score", "DESC": "Soccer match scoreboard with goal tracking",
                "TEAM1": "Barcelona", "TEAM1_VAR": "barca-goals", "SCORE1": "0",
                "TEAM2": "Real Madrid", "TEAM2_VAR": "madrid-goals", "SCORE2": "0",
                "PERIOD": "1st Half", "INC": "1", "CLR1": "#a21caf", "CLR2": "#1d4ed8"}),
    ("tennis", {"TITLE": "Tennis Score", "DESC": "Tennis match point tracker",
                "TEAM1": "Player A", "TEAM1_VAR": "player-a", "SCORE1": "0",
                "TEAM2": "Player B", "TEAM2_VAR": "player-b", "SCORE2": "0",
                "PERIOD": "Set 1", "INC": "15", "CLR1": "#0d9488", "CLR2": "#dc2626"}),
    ("football", {"TITLE": "Football Score", "DESC": "Football game scoreboard with touchdowns",
                  "TEAM1": "Eagles", "TEAM1_VAR": "eagles-pts", "SCORE1": "0",
                  "TEAM2": "Chiefs", "TEAM2_VAR": "chiefs-pts", "SCORE2": "0",
                  "PERIOD": "1st Quarter", "INC": "7", "CLR1": "#065f46", "CLR2": "#b91c1c"}),
    ("hockey", {"TITLE": "Hockey Score", "DESC": "Ice hockey game scoreboard",
                "TEAM1": "Bruins", "TEAM1_VAR": "bruins-goals", "SCORE1": "0",
                "TEAM2": "Rangers", "TEAM2_VAR": "rangers-goals", "SCORE2": "0",
                "PERIOD": "Period 1", "INC": "1", "CLR1": "#ca8a04", "CLR2": "#1e40af"}),
]:
    ex(f"gen-score-{n}.naze", cfg["DESC"], fill(SCORE_T, cfg))

ex("gen-score-leaderboard.naze", "Tournament leaderboard with rankings",
   """-- Tournament leaderboard
app "Tournament Leaderboard" {
  state players = [{name: "Magnus", points: "28", wins: "9"}, {name: "Hikaru", points: "24", wins: "7"}, {name: "Ding", points: "22", wins: "6"}, {name: "Fabiano", points: "20", wins: "5"}]

  column padding: 20px, gap: 12px {
    heading "Tournament Leaderboard"

    row padding: 8px, gap: 16px {
      text "Rank" font-weight: bold, color: #64748b
      text "Player" font-weight: bold, color: #64748b
      text "Points" font-weight: bold, color: #64748b
      text "Wins" font-weight: bold, color: #64748b
    }

    separator

    each player in players | sort-by points {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 16px {
        text "{player.name}" font-weight: bold
        text "{player.points} pts" color: #2563eb
        text "{player.wins} W" color: #16a34a
      }
    }
  }
}""")

ex("gen-score-match-history.naze", "Match history with results log",
   """-- Match history
app "Match History" {
  state matches = [{teams: "Red v Blue", result: "3-1", date: "Feb 15"}, {teams: "Green v Yellow", result: "2-2", date: "Feb 14"}, {teams: "Red v Green", result: "1-0", date: "Feb 13"}]
  computed match-total = matches | count

  column padding: 20px, gap: 16px {
    heading "Match History"
    text "{match-total} recent matches" color: #64748b

    each m in matches {
      row padding: 12px, color: #f3f4f6, radius: 8px, gap: 12px {
        text "{m.teams}" font-weight: bold
        text "{m.result}" font-size: 20px, color: #2563eb
        text "{m.date}" color: #64748b, font-size: 12px
      }
    }
  }
}""")

ex("gen-score-stats.naze", "Player statistics dashboard with key metrics",
   """-- Player stats
app "Player Stats" {
  state player = "Alex Johnson"
  state games = 42
  state goals = 18
  state assists = 12
  computed points-per-game = goals + assists

  column padding: 20px, gap: 16px {
    heading "{player}"
    text "Season Statistics" color: #64748b

    grid columns: 2, gap: 12px {
      rect padding: 12px, color: #eff6ff, radius: 8px {
        text "{games}" font-size: 28px, color: #2563eb
        text "Games" color: #64748b, font-size: 12px
      }
      rect padding: 12px, color: #ecfdf5, radius: 8px {
        text "{goals}" font-size: 28px, color: #16a34a
        text "Goals" color: #64748b, font-size: 12px
      }
      rect padding: 12px, color: #fef3c7, radius: 8px {
        text "{assists}" font-size: 28px, color: #ca8a04
        text "Assists" color: #64748b, font-size: 12px
      }
      rect padding: 12px, color: #faf5ff, radius: 8px {
        text "{points-per-game}" font-size: 28px, color: #7c3aed
        text "Total G+A" color: #64748b, font-size: 12px
      }
    }
  }
}""")

ex("gen-score-live-timer.naze", "Live match timer with score tracking",
   """-- Live match timer
app "Live Match" {
  state elapsed = 0
  state home = 0
  state away = 0

  timer clock: every 1s {
    set elapsed = elapsed + 1
  }

  column padding: 20px, gap: 16px {
    heading "Live Match"
    text "{elapsed} seconds" font-size: 24px, color: #dc2626

    row gap: 32px {
      column gap: 4px {
        text "HOME" font-weight: bold
        text "{home}" font-size: 48px, color: #2563eb
        rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
          text "Goal!" color: #ffffff
          on click: set home = home + 1
        }
      }
      column gap: 4px {
        text "AWAY" font-weight: bold
        text "{away}" font-size: 48px, color: #dc2626
        rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
          text "Goal!" color: #ffffff
          on click: set away = away + 1
        }
      }
    }
  }
}""")

ex("gen-score-bracket.naze", "Tournament bracket display with match pairings",
   """-- Tournament bracket
app "Tournament Bracket" {
  state round = "Quarter Finals"
  state matches = [{pair: "Alpha vs Beta", winner: "TBD"}, {pair: "Gamma vs Delta", winner: "TBD"}, {pair: "Epsilon vs Zeta", winner: "TBD"}, {pair: "Eta vs Theta", winner: "TBD"}]

  column padding: 20px, gap: 16px {
    heading "Tournament Bracket"
    text "{round}" font-size: 18px, color: #6366f1

    each m in matches {
      rect padding: 12px, color: #f8fafc, radius: 8px {
        text "{m.pair}" font-weight: bold
        text "Winner: {m.winner}" color: #64748b
      }
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: #6366f1, radius: 4px {
        text "Semi Finals" color: #ffffff, font-size: 12px
        on click: set round = "Semi Finals"
      }
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "Finals" color: #ffffff, font-size: 12px
        on click: set round = "Finals"
      }
    }
  }
}""")


# ─── Generator: Playlists (gen-playlist-*) ───────────────────────────────────

PLAYLIST_T = """-- __DESC__
app "__TITLE__" {
  state tracks = [__ITEMS__]
  state current-track = 0
  state playing = false
  computed track-count = tracks | count

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{track-count} __LABEL__" color: #64748b

    if playing {
      text "Now Playing" color: __CLR__, font-weight: bold
    }

    each track in tracks {
      row padding: 10px, color: __BG__, radius: 8px, gap: 8px {
        text "{track.__F1__}" font-weight: bold
        text "{track.__F2__}" color: #64748b
        text "{track.__F3__}" color: __CLR__, font-size: 12px
      }
    }

    row gap: 8px {
      rect width: 60px, height: 36px, color: __CLR__, radius: 4px {
        text "Prev" color: #ffffff, font-size: 12px
        on click: set current-track = current-track - 1
      }
      rect width: 60px, height: 36px, color: __CLR__, radius: 4px {
        text "Play" color: #ffffff, font-size: 12px
        on click: set playing = true
      }
      rect width: 60px, height: 36px, color: __CLR__, radius: 4px {
        text "Next" color: #ffffff, font-size: 12px
        on click: set current-track = current-track + 1
      }
    }
  }
}"""

for n, cfg in [
    ("rock", {"TITLE": "Rock Classics", "DESC": "Rock playlist with track listing and controls",
              "ITEMS": '{title: "Bohemian Rhapsody", artist: "Queen", duration: "5:55"}, {title: "Stairway to Heaven", artist: "Led Zeppelin", duration: "8:02"}, {title: "Hotel California", artist: "Eagles", duration: "6:30"}',
              "LABEL": "tracks", "F1": "title", "F2": "artist", "F3": "duration", "BG": "#fef3c7", "CLR": "#b45309"}),
    ("jazz", {"TITLE": "Jazz Essentials", "DESC": "Jazz playlist with classic standards",
              "ITEMS": '{title: "Take Five", artist: "Dave Brubeck", duration: "5:24"}, {title: "So What", artist: "Miles Davis", duration: "9:22"}, {title: "My Favorite Things", artist: "John Coltrane", duration: "13:41"}',
              "LABEL": "tracks", "F1": "title", "F2": "artist", "F3": "duration", "BG": "#eff6ff", "CLR": "#1d4ed8"}),
    ("electronic", {"TITLE": "Electronic Mix", "DESC": "Electronic music playlist with BPM info",
                    "ITEMS": '{title: "Strobe", artist: "Deadmau5", duration: "10:37"}, {title: "Midnight City", artist: "M83", duration: "4:03"}, {title: "Windowlicker", artist: "Aphex Twin", duration: "6:07"}',
                    "LABEL": "tracks", "F1": "title", "F2": "artist", "F3": "duration", "BG": "#fdf2f8", "CLR": "#a21caf"}),
    ("hiphop", {"TITLE": "Hip Hop Hits", "DESC": "Hip hop playlist with popular tracks",
                "ITEMS": '{title: "Lose Yourself", artist: "Eminem", duration: "5:26"}, {title: "Alright", artist: "Kendrick Lamar", duration: "3:39"}, {title: "Juicy", artist: "Notorious B.I.G.", duration: "5:02"}',
                "LABEL": "tracks", "F1": "title", "F2": "artist", "F3": "duration", "BG": "#fefce8", "CLR": "#ca8a04"}),
    ("classical", {"TITLE": "Classical Collection", "DESC": "Classical music playlist with composers",
                   "ITEMS": '{title: "Moonlight Sonata", artist: "Beethoven", duration: "15:00"}, {title: "Four Seasons", artist: "Vivaldi", duration: "11:28"}, {title: "Clair de Lune", artist: "Debussy", duration: "5:00"}',
                   "LABEL": "pieces", "F1": "title", "F2": "artist", "F3": "duration", "BG": "#f8fafc", "CLR": "#475569"}),
]:
    ex(f"gen-playlist-{n}.naze", cfg["DESC"], fill(PLAYLIST_T, cfg))

ex("gen-playlist-queue.naze", "Playlist queue with now playing indicator",
   """-- Playlist queue
app "Up Next" {
  state now-playing = "Bohemian Rhapsody"
  state queue = [{title: "Imagine", artist: "John Lennon"}, {title: "Yesterday", artist: "The Beatles"}, {title: "Purple Rain", artist: "Prince"}]
  computed queue-size = queue | count

  column padding: 20px, gap: 16px {
    heading "Up Next"

    rect padding: 16px, color: #6366f1, radius: 12px {
      text "Now Playing" color: #c7d2fe, font-size: 12px
      text "{now-playing}" color: #ffffff, font-size: 20px, font-weight: bold
    }

    text "{queue-size} tracks in queue" color: #64748b

    each track in queue {
      row padding: 10px, color: #f8fafc, radius: 8px, gap: 8px {
        text "{track.title}" font-weight: bold
        text "{track.artist}" color: #64748b
      }
    }
  }
}""")

ex("gen-playlist-podcast.naze", "Podcast episode playlist with episode descriptions",
   """-- Podcast playlist
app "Podcast Episodes" {
  state episodes = [{title: "Episode 42: The Future", desc: "We discuss what comes next", duration: "45 min"}, {title: "Episode 41: Deep Dive", desc: "Technical deep dive into systems", duration: "38 min"}, {title: "Episode 40: Interview", desc: "Guest interview with industry leaders", duration: "52 min"}]
  computed ep-count = episodes | count

  column padding: 20px, gap: 16px {
    heading "Podcast Episodes"
    text "{ep-count} episodes" color: #64748b

    each ep in episodes {
      rect padding: 16px, color: #f8fafc, radius: 8px {
        row gap: 8px {
          rect width: 48px, height: 48px, color: #8b5cf6, radius: 8px {
            text "P" color: #ffffff, font-size: 20px
          }
          column gap: 4px {
            text "{ep.title}" font-weight: bold
            text "{ep.desc}" color: #64748b, font-size: 14px
            text "{ep.duration}" color: #8b5cf6, font-size: 12px
          }
        }
      }
    }
  }
}""")

ex("gen-playlist-video.naze", "Video playlist with thumbnail placeholders",
   """-- Video playlist
app "Watch Later" {
  state videos = [{title: "Learn Rust in 10 Min", channel: "Code Academy", views: "1.2M"}, {title: "Building a Compiler", channel: "Low Level", views: "890K"}, {title: "UI Design Tips", channel: "Design Hub", views: "2.1M"}]

  column padding: 20px, gap: 16px {
    heading "Watch Later"

    each vid in videos {
      row padding: 8px, color: #f8fafc, radius: 8px, gap: 12px {
        rect width: 120px, height: 68px, color: #1e293b, radius: 4px {
          text "VIDEO" color: #94a3b8, font-size: 12px
        }
        column gap: 4px {
          text "{vid.title}" font-weight: bold
          text "{vid.channel}" color: #64748b, font-size: 14px
          text "{vid.views} views" color: #94a3b8, font-size: 12px
        }
      }
    }
  }
}""")

ex("gen-playlist-create.naze", "Create new playlist form with name and tracks",
   """-- Create playlist
app "New Playlist" {
  state playlist-name = ""
  state track-to-add = ""
  state created = false

  column padding: 20px, gap: 16px {
    heading "Create Playlist"

    input bind: playlist-name, placeholder: "Playlist name"
    input bind: track-to-add, placeholder: "Add a track..."

    rect width: 140px, height: 40px, color: #16a34a, radius: 8px {
      text "Create Playlist" color: #ffffff
      on click: set created = true
    }

    if created {
      text "Playlist created!" color: #16a34a, font-size: 18px
    }
  }
}""")

ex("gen-playlist-shuffle.naze", "Playlist with shuffle and repeat mode toggles",
   """-- Playlist controls
app "Player Controls" {
  state mode = "normal"
  state volume = 80
  state track-pos = 0

  timer progress: every 1s {
    set track-pos = track-pos + 1
  }

  column padding: 20px, gap: 16px {
    heading "Player Controls"
    text "Position: {track-pos}s" font-size: 18px, color: #64748b
    text "Volume: {volume}%" color: #64748b

    match mode {
      "shuffle": text "Shuffle Mode" color: #16a34a, font-weight: bold
      "repeat": text "Repeat Mode" color: #f59e0b, font-weight: bold
      _: text "Normal Mode" color: #64748b
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Shuffle" color: #ffffff, font-size: 12px
        on click: set mode = "shuffle"
      }
      rect width: 80px, height: 36px, color: #f59e0b, radius: 4px {
        text "Repeat" color: #ffffff, font-size: 12px
        on click: set mode = "repeat"
      }
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "Normal" color: #ffffff, font-size: 12px
        on click: set mode = "normal"
      }
    }
  }
}""")


# ─── Generator: Address / Location (gen-address-*) ───────────────────────────

ADDRESS_FORM_T = """-- __DESC__
app "__TITLE__" {
  state __F1__ = ""
  state __F2__ = ""
  state __F3__ = ""
  state __F4__ = ""
  state saved = false

  column padding: 20px, gap: 12px {
    heading "__TITLE__"

    input bind: __F1__, placeholder: "__P1__"
    input bind: __F2__, placeholder: "__P2__"
    input bind: __F3__, placeholder: "__P3__"
    input bind: __F4__, placeholder: "__P4__"

    rect width: 120px, height: 40px, color: __CLR__, radius: 8px {
      text "__BTN__" color: #ffffff
      on click: set saved = true
    }

    if saved {
      text "__MSG__" color: #16a34a
    }
  }
}"""

for n, cfg in [
    ("shipping", {"TITLE": "Shipping Address", "DESC": "Shipping address form with street and zip",
                  "F1": "street", "P1": "Street Address", "F2": "city", "P2": "City",
                  "F3": "state-name", "P3": "State", "F4": "zip", "P4": "ZIP Code",
                  "BTN": "Save", "CLR": "#2563eb", "MSG": "Address saved!"}),
    ("billing", {"TITLE": "Billing Address", "DESC": "Billing address form for payment",
                 "F1": "billing-street", "P1": "Billing Address", "F2": "billing-city", "P2": "City",
                 "F3": "billing-state", "P3": "State/Province", "F4": "billing-zip", "P4": "Postal Code",
                 "BTN": "Continue", "CLR": "#16a34a", "MSG": "Billing address confirmed!"}),
    ("office", {"TITLE": "Office Location", "DESC": "Office address entry form",
                "F1": "company", "P1": "Company Name", "F2": "floor", "P2": "Floor / Suite",
                "F3": "building", "P3": "Building Name", "F4": "address", "P4": "Full Address",
                "BTN": "Register", "CLR": "#6366f1", "MSG": "Office registered!"}),
    ("delivery", {"TITLE": "Delivery Details", "DESC": "Delivery address with instructions",
                  "F1": "recipient", "P1": "Recipient Name", "F2": "delivery-address", "P2": "Delivery Address",
                  "F3": "phone", "P3": "Phone Number", "F4": "instructions", "P4": "Delivery instructions",
                  "BTN": "Submit", "CLR": "#f59e0b", "MSG": "Delivery info saved!"}),
    ("event", {"TITLE": "Event Venue", "DESC": "Event venue location form",
               "F1": "venue-name", "P1": "Venue Name", "F2": "venue-address", "P2": "Venue Address",
               "F3": "capacity", "P3": "Capacity", "F4": "contact", "P4": "Contact Person",
               "BTN": "Add Venue", "CLR": "#a21caf", "MSG": "Venue added!"}),
]:
    ex(f"gen-address-{n}.naze", cfg["DESC"], fill(ADDRESS_FORM_T, cfg))

ex("gen-address-card.naze", "Address card display with formatted location",
   """-- Address card
app "Contact Card" {
  state name = "Acme Corp"
  state street = "123 Innovation Drive"
  state city = "San Francisco"
  state state-abbr = "CA"
  state zip = "94105"

  column padding: 20px, gap: 16px {
    heading "Contact Card"

    rect padding: 20px, color: #f8fafc, radius: 12px {
      text "{name}" font-weight: bold, font-size: 20px
      separator
      text "{street}" color: #374151
      text "{city}, {state-abbr} {zip}" color: #374151
    }
  }
}""")

ex("gen-address-multi.naze", "Multiple saved addresses with selection",
   """-- Address book
app "My Addresses" {
  state addresses = [{label: "Home", addr: "456 Oak Street, Portland OR"}, {label: "Work", addr: "789 Tech Blvd, Seattle WA"}, {label: "Mom", addr: "321 Maple Ave, Denver CO"}]
  state selected = "none"

  column padding: 20px, gap: 16px {
    heading "My Addresses"

    each a in addresses {
      rect padding: 12px, color: #f8fafc, radius: 8px {
        text "{a.label}" font-weight: bold, color: #2563eb
        text "{a.addr}" color: #374151
        rect width: 80px, height: 32px, color: #e2e8f0, radius: 4px {
          text "Select"
          on click: set selected = a.label
        }
      }
    }

    if selected != "none" {
      text "Selected: {selected}" color: #16a34a, font-weight: bold
    }
  }
}""")

ex("gen-address-search.naze", "Address search with input and results",
   """-- Address search
app "Find Address" {
  state query = ""
  state searched = false

  column padding: 20px, gap: 16px {
    heading "Address Lookup"

    input bind: query, placeholder: "Enter address or ZIP..."

    rect width: 100px, height: 40px, color: #2563eb, radius: 8px {
      text "Search" color: #ffffff
      on click: set searched = true
    }

    if searched {
      text "Results for: {query}" font-weight: bold
      rect padding: 12px, color: #ecfdf5, radius: 8px {
        text "123 Main St, Anytown, USA"
        text "ZIP: 90210" color: #64748b
      }
      rect padding: 12px, color: #ecfdf5, radius: 8px {
        text "456 Elm St, Anytown, USA"
        text "ZIP: 90211" color: #64748b
      }
    }
  }
}""")

ex("gen-address-shipping-label.naze", "Printable shipping label layout",
   """-- Shipping label
app "Shipping Label" {
  state from-name = "Acme Corp"
  state from-addr = "100 Sender Way, NY 10001"
  state to-name = "Jane Doe"
  state to-addr = "200 Receiver Rd, LA 90001"
  state tracking = "TRACK-12345-XYZ"

  column padding: 20px, gap: 16px {
    heading "Shipping Label"

    rect padding: 20px, color: #ffffff, radius: 8px {
      text "FROM:" font-size: 12px, color: #64748b
      text "{from-name}" font-weight: bold
      text "{from-addr}" color: #374151

      spacer

      text "TO:" font-size: 12px, color: #64748b
      text "{to-name}" font-weight: bold, font-size: 18px
      text "{to-addr}" color: #374151

      separator

      text "Tracking: {tracking}" color: #2563eb, font-size: 12px
    }
  }
}""")

ex("gen-address-map-list.naze", "Location list with coordinates and distances",
   """-- Location list
app "Nearby Locations" {
  state locations = [{name: "Central Park", dist: "0.5 mi", type: "Park"}, {name: "City Library", dist: "1.2 mi", type: "Library"}, {name: "Train Station", dist: "0.8 mi", type: "Transit"}, {name: "Hospital", dist: "2.1 mi", type: "Medical"}]

  column padding: 20px, gap: 12px {
    heading "Nearby Locations"

    each loc in locations | sort-by dist {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 12px {
        rect width: 40px, height: 40px, color: #6366f1, radius: 20px {
          text "{loc.type}" color: #ffffff, font-size: 8px
        }
        column gap: 2px {
          text "{loc.name}" font-weight: bold
          text "{loc.dist}" color: #64748b
        }
      }
    }
  }
}""")


# ─── Generator: Feature Comparison (gen-compare-*) ───────────────────────────

COMPARE_T = """-- __DESC__
app "__TITLE__" {
  state selected = "none"

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "__SUBTITLE__" color: #64748b

    grid columns: __COLS__, gap: 12px {
      rect padding: 16px, color: __C1__, radius: 8px {
        text "__N1__" font-weight: bold, font-size: 18px
        text "__D1__" color: #64748b
        separator
        text "__F1A__"
        text "__F1B__"
        text "__F1C__"
        rect width: 100px, height: 32px, color: __CLR__, radius: 4px {
          text "Select" color: #ffffff
          on click: set selected = "__N1__"
        }
      }
      rect padding: 16px, color: __C2__, radius: 8px {
        text "__N2__" font-weight: bold, font-size: 18px
        text "__D2__" color: #64748b
        separator
        text "__F2A__"
        text "__F2B__"
        text "__F2C__"
        rect width: 100px, height: 32px, color: __CLR__, radius: 4px {
          text "Select" color: #ffffff
          on click: set selected = "__N2__"
        }
      }
    }

    if selected != "none" {
      text "Selected: {selected}" color: __CLR__, font-weight: bold
    }
  }
}"""

for n, cfg in [
    ("plans", {"TITLE": "Plan Comparison", "DESC": "Compare subscription plans side by side",
               "SUBTITLE": "Choose the right plan", "COLS": "2",
               "N1": "Basic", "D1": "$9/mo", "F1A": "5 Projects", "F1B": "1GB Storage", "F1C": "Email Support",
               "N2": "Pro", "D2": "$29/mo", "F2A": "Unlimited Projects", "F2B": "100GB Storage", "F2C": "Priority Support",
               "C1": "#f0f9ff", "C2": "#ecfdf5", "CLR": "#2563eb"}),
    ("phones", {"TITLE": "Phone Comparison", "DESC": "Compare phone specifications side by side",
                "SUBTITLE": "Which phone is right for you?", "COLS": "2",
                "N1": "Phone X", "D1": "$999", "F1A": "6.1 inch display", "F1B": "128GB Storage", "F1C": "12MP Camera",
                "N2": "Phone Y", "D2": "$799", "F2A": "6.5 inch display", "F2B": "256GB Storage", "F2C": "48MP Camera",
                "C1": "#fefce8", "C2": "#faf5ff", "CLR": "#7c3aed"}),
    ("laptops", {"TITLE": "Laptop Comparison", "DESC": "Compare laptop specs and pricing",
                 "SUBTITLE": "Find your perfect laptop", "COLS": "2",
                 "N1": "Ultrabook", "D1": "$1299", "F1A": "14 inch, 2.2 lbs", "F1B": "16GB RAM", "F1C": "512GB SSD",
                 "N2": "Workstation", "D2": "$1899", "F2A": "16 inch, 4.5 lbs", "F2B": "32GB RAM", "F2C": "1TB SSD",
                 "C1": "#f8fafc", "C2": "#eff6ff", "CLR": "#0284c7"}),
    ("frameworks", {"TITLE": "Framework Comparison", "DESC": "Compare web framework features",
                    "SUBTITLE": "Choose your stack", "COLS": "2",
                    "N1": "Framework A", "D1": "Open Source", "F1A": "Fast builds", "F1B": "Small bundle", "F1C": "Growing community",
                    "N2": "Framework B", "D2": "Open Source", "F2A": "Mature ecosystem", "F2B": "Large bundle", "F2C": "Huge community",
                    "C1": "#ecfdf5", "C2": "#fef3c7", "CLR": "#16a34a"}),
    ("hosting", {"TITLE": "Hosting Comparison", "DESC": "Compare cloud hosting providers",
                 "SUBTITLE": "Compare providers", "COLS": "2",
                 "N1": "Cloud A", "D1": "From $5/mo", "F1A": "99.9% Uptime", "F1B": "Auto scaling", "F1C": "Free SSL",
                 "N2": "Cloud B", "D2": "From $10/mo", "F2A": "99.99% Uptime", "F2B": "Global CDN", "F2C": "DDoS Protection",
                 "C1": "#fdf2f8", "C2": "#f0f9ff", "CLR": "#db2777"}),
]:
    ex(f"gen-compare-{n}.naze", cfg["DESC"], fill(COMPARE_T, cfg))

ex("gen-compare-before-after.naze", "Before and after comparison view",
   """-- Before/After comparison
app "Before and After" {
  state view = "before"

  column padding: 20px, gap: 16px {
    heading "Before and After"

    match view {
      "before": rect padding: 20px, color: #fef2f2, radius: 8px {
        text "Before" font-weight: bold, font-size: 20px
        text "Old design with cluttered layout"
        text "Slow load times" color: #dc2626
        text "Poor accessibility" color: #dc2626
      }
      _: rect padding: 20px, color: #f0fdf4, radius: 8px {
        text "After" font-weight: bold, font-size: 20px
        text "Clean and modern interface"
        text "Fast performance" color: #16a34a
        text "WCAG compliant" color: #16a34a
      }
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #ef4444, radius: 4px {
        text "Before" color: #ffffff
        on click: set view = "before"
      }
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "After" color: #ffffff
        on click: set view = "after"
      }
    }
  }
}""")

ex("gen-compare-pros-cons.naze", "Pros and cons list for a product",
   """-- Pros and cons
app "Pros and Cons" {
  state pros = [{item: "Fast performance"}, {item: "Great battery life"}, {item: "Beautiful display"}]
  state cons = [{item: "Expensive"}, {item: "No expandable storage"}]

  column padding: 20px, gap: 16px {
    heading "Product Assessment"

    grid columns: 2, gap: 16px {
      rect padding: 16px, color: #f0fdf4, radius: 8px {
        text "Pros" font-weight: bold, color: #16a34a, font-size: 18px
        each p in pros {
          text "+ {p.item}" color: #16a34a
        }
      }
      rect padding: 16px, color: #fef2f2, radius: 8px {
        text "Cons" font-weight: bold, color: #dc2626, font-size: 18px
        each c in cons {
          text "- {c.item}" color: #dc2626
        }
      }
    }
  }
}""")

ex("gen-compare-pricing-tier.naze", "Three-tier pricing comparison table",
   """-- Three-tier pricing
app "Pricing" {
  state selected-plan = "none"

  column padding: 20px, gap: 16px {
    heading "Choose Your Plan"

    grid columns: 3, gap: 12px {
      rect padding: 16px, color: #f8fafc, radius: 8px {
        text "Starter" font-weight: bold
        text "$0/mo" font-size: 24px, color: #64748b
        text "1 project"
        text "500MB storage"
        rect width: 80px, height: 32px, color: #64748b, radius: 4px {
          text "Free" color: #ffffff
          on click: set selected-plan = "starter"
        }
      }
      rect padding: 16px, color: #eff6ff, radius: 8px {
        text "Growth" font-weight: bold
        text "$19/mo" font-size: 24px, color: #2563eb
        text "10 projects"
        text "50GB storage"
        rect width: 80px, height: 32px, color: #2563eb, radius: 4px {
          text "Start" color: #ffffff
          on click: set selected-plan = "growth"
        }
      }
      rect padding: 16px, color: #fefce8, radius: 8px {
        text "Enterprise" font-weight: bold
        text "$99/mo" font-size: 24px, color: #ca8a04
        text "Unlimited projects"
        text "1TB storage"
        rect width: 80px, height: 32px, color: #ca8a04, radius: 4px {
          text "Contact" color: #ffffff
          on click: set selected-plan = "enterprise"
        }
      }
    }

    if selected-plan != "none" {
      text "Selected: {selected-plan}" color: #2563eb, font-weight: bold
    }
  }
}""")

ex("gen-compare-specs.naze", "Technical specifications comparison table",
   """-- Specs comparison
app "Spec Sheet" {
  state specs = [{feature: "CPU", option-a: "8 cores", option-b: "12 cores"}, {feature: "RAM", option-a: "16 GB", option-b: "32 GB"}, {feature: "Storage", option-a: "512 GB", option-b: "1 TB"}, {feature: "Battery", option-a: "10 hrs", option-b: "8 hrs"}]

  column padding: 20px, gap: 12px {
    heading "Spec Comparison"

    row padding: 8px, gap: 16px {
      text "Feature" font-weight: bold, color: #64748b
      text "Model A" font-weight: bold, color: #2563eb
      text "Model B" font-weight: bold, color: #16a34a
    }

    separator

    each s in specs {
      row padding: 8px, color: #f8fafc, radius: 4px, gap: 16px {
        text "{s.feature}" font-weight: bold
        text "{s.option-a}" color: #2563eb
        text "{s.option-b}" color: #16a34a
      }
    }
  }
}""")

ex("gen-compare-ratings.naze", "Multi-criteria rating comparison",
   """-- Rating comparison
app "Rating Comparison" {
  state product-a = "Widget Pro"
  state product-b = "Widget Lite"
  state perf-a = 9
  state perf-b = 7
  state design-a = 8
  state design-b = 9
  state value-a = 6
  state value-b = 9

  column padding: 20px, gap: 16px {
    heading "Rating Comparison"

    grid columns: 3, gap: 8px {
      text "Criteria" font-weight: bold, color: #64748b
      text "{product-a}" font-weight: bold, color: #2563eb
      text "{product-b}" font-weight: bold, color: #16a34a

      text "Performance"
      text "{perf-a}/10" color: #2563eb
      text "{perf-b}/10" color: #16a34a

      text "Design"
      text "{design-a}/10" color: #2563eb
      text "{design-b}/10" color: #16a34a

      text "Value"
      text "{value-a}/10" color: #2563eb
      text "{value-b}/10" color: #16a34a
    }
  }
}""")


# ─── Generator: Countdown Timers (gen-countdown-*) ───────────────────────────

COUNTDOWN_T = """-- __DESC__
app "__TITLE__" {
  state __VAR__ = __INIT__
  state event-name = "__EVENT__"

  timer tick: every __INTERVAL__ {
    set __VAR__ = __VAR__ - 1
  }

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{event-name}" color: #64748b

    rect padding: 24px, color: __BG__, radius: 12px {
      text "{__VAR__}" font-size: 64px, color: __CLR__
      text "__UNIT__" color: __CLR__
    }

    if __VAR__ == 0 {
      text "__DONE_MSG__" font-size: 20px, color: #16a34a, font-weight: bold
    }

    rect width: 100px, height: 36px, color: #64748b, radius: 4px {
      text "Reset" color: #ffffff
      on click: set __VAR__ = __INIT__
    }
  }
}"""

for n, cfg in [
    ("launch", {"TITLE": "Launch Countdown", "DESC": "Product launch countdown timer",
                "VAR": "seconds-left", "INIT": "60", "EVENT": "Product Launch",
                "INTERVAL": "1s", "UNIT": "seconds to launch", "BG": "#1e293b",
                "CLR": "#f59e0b", "DONE_MSG": "We are LIVE!"}),
    ("newyear", {"TITLE": "New Year Countdown", "DESC": "New Year countdown in seconds",
                 "VAR": "secs", "INIT": "120", "EVENT": "Happy New Year 2027",
                 "INTERVAL": "1s", "UNIT": "seconds remaining", "BG": "#1e1b4b",
                 "CLR": "#c084fc", "DONE_MSG": "Happy New Year!"}),
    ("sale", {"TITLE": "Flash Sale", "DESC": "Flash sale countdown with urgency",
              "VAR": "sale-time", "INIT": "300", "EVENT": "Flash Sale Ends Soon",
              "INTERVAL": "1s", "UNIT": "seconds left to save", "BG": "#fef2f2",
              "CLR": "#dc2626", "DONE_MSG": "Sale has ended!"}),
    ("exam", {"TITLE": "Exam Timer", "DESC": "Exam countdown with minutes remaining",
              "VAR": "exam-min", "INIT": "45", "EVENT": "Final Examination",
              "INTERVAL": "1min", "UNIT": "minutes remaining", "BG": "#eff6ff",
              "CLR": "#1d4ed8", "DONE_MSG": "Time is up!"}),
    ("break-timer", {"TITLE": "Break Timer", "DESC": "Work break countdown timer",
                     "VAR": "break-secs", "INIT": "300", "EVENT": "Take a Break",
                     "INTERVAL": "1s", "UNIT": "seconds of break left", "BG": "#ecfdf5",
                     "CLR": "#059669", "DONE_MSG": "Break over!"}),
]:
    ex(f"gen-countdown-{n}.naze", cfg["DESC"], fill(COUNTDOWN_T, cfg))

ex("gen-countdown-multi.naze", "Multiple countdowns for different events",
   """-- Multiple countdowns
app "Event Countdowns" {
  state events = [{name: "Conference", days: "15"}, {name: "Birthday", days: "32"}, {name: "Vacation", days: "45"}, {name: "Deadline", days: "7"}]

  column padding: 20px, gap: 16px {
    heading "Upcoming Events"

    each evt in events | sort-by days {
      row padding: 12px, color: #f8fafc, radius: 8px, gap: 12px {
        rect width: 64px, height: 64px, color: #6366f1, radius: 8px {
          text "{evt.days}" color: #ffffff, font-size: 24px
        }
        column gap: 4px {
          text "{evt.name}" font-weight: bold, font-size: 16px
          text "{evt.days} days away" color: #64748b
        }
      }
    }
  }
}""")

ex("gen-countdown-flip.naze", "Flip clock style countdown display",
   """-- Flip clock countdown
app "Flip Clock" {
  state hours = 2
  state minutes = 30
  state seconds = 0

  timer tick: every 1s {
    set seconds = seconds - 1
  }

  column padding: 20px, gap: 16px {
    heading "Countdown Clock"

    row gap: 8px {
      rect width: 80px, height: 80px, color: #1e293b, radius: 8px {
        text "{hours}" color: #ffffff, font-size: 36px
        text "HRS" color: #94a3b8, font-size: 10px
      }
      text ":" font-size: 36px, color: #64748b
      rect width: 80px, height: 80px, color: #1e293b, radius: 8px {
        text "{minutes}" color: #ffffff, font-size: 36px
        text "MIN" color: #94a3b8, font-size: 10px
      }
      text ":" font-size: 36px, color: #64748b
      rect width: 80px, height: 80px, color: #1e293b, radius: 8px {
        text "{seconds}" color: #ffffff, font-size: 36px
        text "SEC" color: #94a3b8, font-size: 10px
      }
    }
  }
}""")

ex("gen-countdown-progress.naze", "Countdown with visual progress indicator",
   """-- Countdown with progress
app "Progress Countdown" {
  state total = 100
  state remaining = 100

  timer tick: every 1s {
    set remaining = remaining - 1
  }

  column padding: 20px, gap: 16px {
    heading "Task Countdown"
    text "{remaining} of {total} seconds" color: #64748b

    rect width: 300px, height: 24px, color: #e2e8f0, radius: 12px {
      rect width: 200px, height: 24px, color: #2563eb, radius: 12px
    }

    text "{remaining} seconds left" font-size: 24px, color: #2563eb

    row gap: 8px {
      rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
        text "Reset" color: #ffffff
        on click: set remaining = 100
      }
    }
  }
}""")

ex("gen-countdown-pomodoro.naze", "Pomodoro timer with work and break cycles",
   """-- Pomodoro timer
app "Pomodoro" {
  state time-left = 1500
  state mode = "work"
  state sessions = 0

  timer pomodoro: every 1s {
    set time-left = time-left - 1
  }

  column padding: 20px, gap: 16px {
    heading "Pomodoro Timer"

    match mode {
      "work": text "Focus Time" color: #dc2626, font-weight: bold, font-size: 20px
      _: text "Break Time" color: #16a34a, font-weight: bold, font-size: 20px
    }

    text "{time-left} seconds" font-size: 48px, color: #1e293b
    text "Sessions completed: {sessions}" color: #64748b

    row gap: 8px {
      rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
        text "Work" color: #ffffff
        on click: set mode = "work"
      }
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Break" color: #ffffff
        on click: set mode = "break"
      }
      rect width: 80px, height: 36px, color: #64748b, radius: 4px {
        text "Reset" color: #ffffff
        on click: set time-left = 1500
      }
    }
  }
}""")

ex("gen-countdown-auction.naze", "Auction countdown with current bid display",
   """-- Auction countdown
app "Live Auction" {
  state time-remaining = 180
  state current-bid = 500
  state bid-count = 12

  timer auction: every 1s {
    set time-remaining = time-remaining - 1
  }

  column padding: 20px, gap: 16px {
    heading "Live Auction"

    rect padding: 20px, color: #fef3c7, radius: 12px {
      text "{time-remaining} seconds left" font-size: 28px, color: #b45309
    }

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "Current Bid" color: #64748b
      text "${current-bid}" font-size: 36px, color: #16a34a, font-weight: bold
      text "{bid-count} bids" color: #64748b
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "+$50" color: #ffffff
        on click: set current-bid = current-bid + 50
      }
      rect width: 80px, height: 36px, color: #2563eb, radius: 4px {
        text "+$100" color: #ffffff
        on click: set current-bid = current-bid + 100
      }
    }
  }
}""")


# ─── Generator: Filter Panels (gen-filter-*) ─────────────────────────────────

FILTER_T = """-- __DESC__
app "__TITLE__" {
  state __F1__ = __V1__
  state __F2__ = "__V2__"
  state results-count = __RC__

  column padding: 20px, gap: 16px {
    heading "__TITLE__"
    text "{results-count} results" color: #64748b

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "Filters" font-weight: bold, font-size: 16px

      text "__LABEL1__" font-weight: bold, color: #374151, font-size: 14px
      __FILTER1__

      text "__LABEL2__" font-weight: bold, color: #374151, font-size: 14px
      __FILTER2__
    }

    rect width: 120px, height: 36px, color: __CLR__, radius: 4px {
      text "Apply Filters" color: #ffffff
      on click: set results-count = __NEW_RC__
    }

    rect width: 100px, height: 36px, color: #e2e8f0, radius: 4px {
      text "Clear All"
      on click: set __F1__ = __V1__
    }
  }
}"""

for n, cfg in [
    ("products", {"TITLE": "Product Filters", "DESC": "Product filter panel with category and price",
                  "F1": "max-price", "V1": "100", "F2": "category", "V2": "all",
                  "RC": "50", "NEW_RC": "12",
                  "LABEL1": "Category", "FILTER1": 'select bind: category {\n      option "All" value: "all"\n      option "Electronics" value: "electronics"\n      option "Clothing" value: "clothing"\n    }',
                  "LABEL2": "Max Price", "FILTER2": 'input bind: max-price, placeholder: "Max price"',
                  "CLR": "#2563eb"}),
    ("jobs", {"TITLE": "Job Filters", "DESC": "Job listing filter panel with role and location",
              "F1": "experience", "V1": "0", "F2": "job-type", "V2": "all",
              "RC": "85", "NEW_RC": "23",
              "LABEL1": "Job Type", "FILTER1": 'select bind: job-type {\n      option "All Types" value: "all"\n      option "Full-time" value: "full"\n      option "Part-time" value: "part"\n      option "Contract" value: "contract"\n    }',
              "LABEL2": "Min Experience (years)", "FILTER2": 'input bind: experience, placeholder: "Years"',
              "CLR": "#16a34a"}),
    ("recipes-filter", {"TITLE": "Recipe Filters", "DESC": "Recipe filter panel with cuisine and cook time",
                        "F1": "max-time", "V1": "60", "F2": "cuisine", "V2": "all",
                        "RC": "120", "NEW_RC": "18",
                        "LABEL1": "Cuisine", "FILTER1": 'select bind: cuisine {\n      option "All" value: "all"\n      option "Italian" value: "italian"\n      option "Asian" value: "asian"\n      option "Mexican" value: "mexican"\n    }',
                        "LABEL2": "Max Cook Time (min)", "FILTER2": 'input bind: max-time, placeholder: "Minutes"',
                        "CLR": "#ea580c"}),
    ("housing", {"TITLE": "Property Filters", "DESC": "Real estate filter with bedrooms and type",
                 "F1": "min-beds", "V1": "1", "F2": "property-type", "V2": "any",
                 "RC": "200", "NEW_RC": "34",
                 "LABEL1": "Property Type", "FILTER1": 'select bind: property-type {\n      option "Any" value: "any"\n      option "House" value: "house"\n      option "Apartment" value: "apt"\n      option "Condo" value: "condo"\n    }',
                 "LABEL2": "Min Bedrooms", "FILTER2": 'input bind: min-beds, placeholder: "Bedrooms"',
                 "CLR": "#7c3aed"}),
    ("events-filter", {"TITLE": "Event Filters", "DESC": "Event filter panel with type and date range",
                       "F1": "month", "V1": "0", "F2": "event-type", "V2": "all",
                       "RC": "65", "NEW_RC": "15",
                       "LABEL1": "Event Type", "FILTER1": 'select bind: event-type {\n      option "All Events" value: "all"\n      option "Conference" value: "conference"\n      option "Workshop" value: "workshop"\n      option "Meetup" value: "meetup"\n    }',
                       "LABEL2": "Month", "FILTER2": 'input bind: month, placeholder: "Month (1-12)"',
                       "CLR": "#ca8a04"}),
]:
    ex(f"gen-filter-{n}.naze", cfg["DESC"], fill(FILTER_T, cfg))

ex("gen-filter-checkbox.naze", "Filter panel with checkbox options",
   """-- Checkbox filter panel
app "Tag Filters" {
  state show-featured = false
  state show-sale = false
  state show-new = false
  state filtered-count = 100

  column padding: 20px, gap: 16px {
    heading "Filter by Tags"
    text "{filtered-count} items" color: #64748b

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "Tags" font-weight: bold, font-size: 16px

      checkbox bind: show-featured, label: "Featured items"
      checkbox bind: show-sale, label: "On sale"
      checkbox bind: show-new, label: "New arrivals"
    }

    rect width: 120px, height: 36px, color: #2563eb, radius: 4px {
      text "Apply" color: #ffffff
      on click: set filtered-count = 25
    }
  }
}""")

ex("gen-filter-search-combo.naze", "Combined search and filter interface",
   """-- Search with filters
app "Search and Filter" {
  state search-query = ""
  state sort-order = "relevance"
  state result-count = 42

  column padding: 20px, gap: 16px {
    heading "Search"

    row gap: 8px {
      input bind: search-query, placeholder: "Search items..."
      rect width: 80px, height: 40px, color: #2563eb, radius: 8px {
        text "Search" color: #ffffff
        on click: set result-count = 10
      }
    }

    row gap: 8px {
      text "Sort by:" color: #64748b
      select bind: sort-order {
        option "Relevance" value: "relevance"
        option "Newest" value: "newest"
        option "Price" value: "price"
      }
    }

    text "{result-count} results found" color: #64748b
  }
}""")

ex("gen-filter-price-range.naze", "Price range filter with min and max inputs",
   """-- Price range filter
app "Price Filter" {
  state min-price = 0
  state max-price = 500
  state showing = 78

  column padding: 20px, gap: 16px {
    heading "Price Range"

    rect padding: 16px, color: #f8fafc, radius: 8px {
      row gap: 12px {
        column gap: 4px {
          text "Min Price" font-size: 12px, color: #64748b
          input bind: min-price, placeholder: "Min $"
        }
        column gap: 4px {
          text "Max Price" font-size: 12px, color: #64748b
          input bind: max-price, placeholder: "Max $"
        }
      }
    }

    text "Showing {showing} items from ${min-price} to ${max-price}" color: #64748b

    rect width: 120px, height: 36px, color: #16a34a, radius: 4px {
      text "Apply Range" color: #ffffff
      on click: set showing = 22
    }
  }
}""")

ex("gen-filter-multi-select.naze", "Multi-criteria filter with category checkboxes and sort",
   """-- Multi-criteria filter
app "Advanced Filters" {
  state show-active = true
  state show-archived = false
  state sort-field = "name"
  state total = 150
  state filtered = 150

  column padding: 20px, gap: 16px {
    heading "Advanced Filters"
    text "{filtered} of {total} items" color: #64748b

    rect padding: 16px, color: #f8fafc, radius: 8px {
      text "Status" font-weight: bold
      checkbox bind: show-active, label: "Active"
      checkbox bind: show-archived, label: "Archived"

      spacer

      text "Sort By" font-weight: bold
      select bind: sort-field {
        option "Name" value: "name"
        option "Date" value: "date"
        option "Status" value: "status"
      }
    }

    row gap: 8px {
      rect width: 100px, height: 36px, color: #2563eb, radius: 4px {
        text "Apply" color: #ffffff
        on click: set filtered = 45
      }
      rect width: 100px, height: 36px, color: #e2e8f0, radius: 4px {
        text "Reset"
        on click: set filtered = 150
      }
    }
  }
}""")

ex("gen-filter-saved.naze", "Saved filter presets with quick apply",
   """-- Saved filter presets
app "Filter Presets" {
  state active-preset = "none"
  state result-count = 200

  column padding: 20px, gap: 16px {
    heading "Filter Presets"
    text "{result-count} results" color: #64748b

    text "Quick Filters" font-weight: bold
    row gap: 8px {
      rect width: 100px, height: 32px, color: #2563eb, radius: 16px {
        text "Top Rated" color: #ffffff, font-size: 12px
        on click: set active-preset = "top-rated"
      }
      rect width: 100px, height: 32px, color: #16a34a, radius: 16px {
        text "New Today" color: #ffffff, font-size: 12px
        on click: set active-preset = "new-today"
      }
      rect width: 100px, height: 32px, color: #f59e0b, radius: 16px {
        text "On Sale" color: #ffffff, font-size: 12px
        on click: set active-preset = "on-sale"
      }
    }

    match active-preset {
      "top-rated": text "Showing top rated items" color: #2563eb
      "new-today": text "Showing items added today" color: #16a34a
      "on-sale": text "Showing sale items" color: #f59e0b
      _: text "No filter applied" color: #64748b
    }

    rect width: 100px, height: 32px, color: #e2e8f0, radius: 4px {
      text "Clear Filter"
      on click: set active-preset = "none"
    }
  }
}""")


def main():
    parser = argparse.ArgumentParser(description="Generate .naze example files")
    parser.add_argument("--limit", type=int, default=0, help="Max examples (0=all)")
    parser.add_argument("--no-validate", action="store_true", help="Skip validation")
    args = parser.parse_args()

    examples = EXAMPLES[: args.limit] if args.limit > 0 else EXAMPLES
    total = len(EXAMPLES)
    print(f"Generating examples... ({len(examples)} of {total})")

    if not args.no_validate:
        build_nazec()

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    valid_count = 0
    failed_count = 0
    start = time.time()

    for i, (filename, description, code) in enumerate(examples, 1):
        filepath = OUTPUT_DIR / filename
        filepath.write_text(code)

        if args.no_validate:
            valid_count += 1
            continue

        ok, stderr = validate(filepath)
        if ok:
            valid_count += 1
            if i % 50 == 0 or i == len(examples):
                print(f"  [{i}/{len(examples)}] ... {valid_count} valid so far")
        else:
            failed_count += 1
            first_line = stderr.split("\n")[0] if stderr else "unknown"
            print(f"  [{i}/{len(examples)}] {filename} FAIL: {first_line}")
            filepath.unlink()

    elapsed = time.time() - start
    print(f"\nDone: {valid_count} valid, {failed_count} failed in {elapsed:.1f}s")
    print(f"Output: {OUTPUT_DIR}/")

    if failed_count > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
