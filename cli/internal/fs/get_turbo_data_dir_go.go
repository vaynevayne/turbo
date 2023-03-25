//go:build go || !rust
// +build go !rust

package fs

import (
	"fmt"

	"github.com/adrg/xdg"
	"github.com/vercel/turbo/cli/internal/turbopath"
)

// GetTurboDataDir returns a directory outside of the repo
// where turbo can store data files related to turbo.
func GetTurboDataDir() turbopath.AbsoluteSystemPath {
	dataHome := AbsoluteSystemPathFromUpstream(xdg.DataHome)
	xx := dataHome.UntypedJoin("turborepo")
	fmt.Printf("[debug] turbodata dir %#v\n", xx)
	return xx
}
