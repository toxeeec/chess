package graph

//go:generate go run github.com/99designs/gqlgen generate

import (
	"github.com/toxeeec/chess/services/go/proto"
)

type Resolver struct {
	ChessClient proto.ChessClient
}
