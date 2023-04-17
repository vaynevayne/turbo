//go:build rust
// +build rust

package cache

import (
	"github.com/vercel/turbo/cli/internal/ffi"
	"github.com/vercel/turbo/cli/internal/turbopath"
)

func (cache *HttpCache) retrieve(hash string) (bool, []turbopath.AnchoredSystemPath, int, error) {
	apiClient := cache.GetAPIClient()
	return ffi.HttpCache__Retrieve(hash, apiClient.GetBaseUrl(), apiClient.GetTimeout(), apiClient.GetVersion(), apiClient.GetToken(), apiClient.GetTeamID(), apiClient.GetTeamSlug(), apiClient.GetUsePreflight(), cache.GetAuthenticator().isEnabled(), cache.repoRoot)
}
