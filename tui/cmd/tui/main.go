package main

import (
    "context"
    "log"

    "github.com/davidlowes/ib_box_spread_full_universal/tui/internal/app"
)

func main() {
    ctx := context.Background()
    if err := app.Run(ctx); err != nil {
        log.Fatalf("tui exited: %v", err)
    }
}

