// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.29.0
// 	protoc        v3.21.12
// source: chess.proto

package proto

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

type MakeMoveRequest struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Move string `protobuf:"bytes,1,opt,name=move,proto3" json:"move,omitempty"`
}

func (x *MakeMoveRequest) Reset() {
	*x = MakeMoveRequest{}
	if protoimpl.UnsafeEnabled {
		mi := &file_chess_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *MakeMoveRequest) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*MakeMoveRequest) ProtoMessage() {}

func (x *MakeMoveRequest) ProtoReflect() protoreflect.Message {
	mi := &file_chess_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use MakeMoveRequest.ProtoReflect.Descriptor instead.
func (*MakeMoveRequest) Descriptor() ([]byte, []int) {
	return file_chess_proto_rawDescGZIP(), []int{0}
}

func (x *MakeMoveRequest) GetMove() string {
	if x != nil {
		return x.Move
	}
	return ""
}

type MakeMoveResponse struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields
}

func (x *MakeMoveResponse) Reset() {
	*x = MakeMoveResponse{}
	if protoimpl.UnsafeEnabled {
		mi := &file_chess_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *MakeMoveResponse) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*MakeMoveResponse) ProtoMessage() {}

func (x *MakeMoveResponse) ProtoReflect() protoreflect.Message {
	mi := &file_chess_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use MakeMoveResponse.ProtoReflect.Descriptor instead.
func (*MakeMoveResponse) Descriptor() ([]byte, []int) {
	return file_chess_proto_rawDescGZIP(), []int{1}
}

type JoinGameRequest struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields
}

func (x *JoinGameRequest) Reset() {
	*x = JoinGameRequest{}
	if protoimpl.UnsafeEnabled {
		mi := &file_chess_proto_msgTypes[2]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *JoinGameRequest) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*JoinGameRequest) ProtoMessage() {}

func (x *JoinGameRequest) ProtoReflect() protoreflect.Message {
	mi := &file_chess_proto_msgTypes[2]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use JoinGameRequest.ProtoReflect.Descriptor instead.
func (*JoinGameRequest) Descriptor() ([]byte, []int) {
	return file_chess_proto_rawDescGZIP(), []int{2}
}

type JoinGameResponse struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Fen    string   `protobuf:"bytes,1,opt,name=fen,proto3" json:"fen,omitempty"`
	Result *float64 `protobuf:"fixed64,2,opt,name=result,proto3,oneof" json:"result,omitempty"`
	Moves  []string `protobuf:"bytes,3,rep,name=moves,proto3" json:"moves,omitempty"`
}

func (x *JoinGameResponse) Reset() {
	*x = JoinGameResponse{}
	if protoimpl.UnsafeEnabled {
		mi := &file_chess_proto_msgTypes[3]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *JoinGameResponse) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*JoinGameResponse) ProtoMessage() {}

func (x *JoinGameResponse) ProtoReflect() protoreflect.Message {
	mi := &file_chess_proto_msgTypes[3]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use JoinGameResponse.ProtoReflect.Descriptor instead.
func (*JoinGameResponse) Descriptor() ([]byte, []int) {
	return file_chess_proto_rawDescGZIP(), []int{3}
}

func (x *JoinGameResponse) GetFen() string {
	if x != nil {
		return x.Fen
	}
	return ""
}

func (x *JoinGameResponse) GetResult() float64 {
	if x != nil && x.Result != nil {
		return *x.Result
	}
	return 0
}

func (x *JoinGameResponse) GetMoves() []string {
	if x != nil {
		return x.Moves
	}
	return nil
}

var File_chess_proto protoreflect.FileDescriptor

var file_chess_proto_rawDesc = []byte{
	0x0a, 0x0b, 0x63, 0x68, 0x65, 0x73, 0x73, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x05, 0x63,
	0x68, 0x65, 0x73, 0x73, 0x22, 0x25, 0x0a, 0x0f, 0x4d, 0x61, 0x6b, 0x65, 0x4d, 0x6f, 0x76, 0x65,
	0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x12, 0x0a, 0x04, 0x6d, 0x6f, 0x76, 0x65, 0x18,
	0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x04, 0x6d, 0x6f, 0x76, 0x65, 0x22, 0x12, 0x0a, 0x10, 0x4d,
	0x61, 0x6b, 0x65, 0x4d, 0x6f, 0x76, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x22,
	0x11, 0x0a, 0x0f, 0x4a, 0x6f, 0x69, 0x6e, 0x47, 0x61, 0x6d, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65,
	0x73, 0x74, 0x22, 0x62, 0x0a, 0x10, 0x4a, 0x6f, 0x69, 0x6e, 0x47, 0x61, 0x6d, 0x65, 0x52, 0x65,
	0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x10, 0x0a, 0x03, 0x66, 0x65, 0x6e, 0x18, 0x01, 0x20,
	0x01, 0x28, 0x09, 0x52, 0x03, 0x66, 0x65, 0x6e, 0x12, 0x1b, 0x0a, 0x06, 0x72, 0x65, 0x73, 0x75,
	0x6c, 0x74, 0x18, 0x02, 0x20, 0x01, 0x28, 0x01, 0x48, 0x00, 0x52, 0x06, 0x72, 0x65, 0x73, 0x75,
	0x6c, 0x74, 0x88, 0x01, 0x01, 0x12, 0x14, 0x0a, 0x05, 0x6d, 0x6f, 0x76, 0x65, 0x73, 0x18, 0x03,
	0x20, 0x03, 0x28, 0x09, 0x52, 0x05, 0x6d, 0x6f, 0x76, 0x65, 0x73, 0x42, 0x09, 0x0a, 0x07, 0x5f,
	0x72, 0x65, 0x73, 0x75, 0x6c, 0x74, 0x32, 0x83, 0x01, 0x0a, 0x05, 0x43, 0x68, 0x65, 0x73, 0x73,
	0x12, 0x3b, 0x0a, 0x08, 0x4d, 0x61, 0x6b, 0x65, 0x4d, 0x6f, 0x76, 0x65, 0x12, 0x16, 0x2e, 0x63,
	0x68, 0x65, 0x73, 0x73, 0x2e, 0x4d, 0x61, 0x6b, 0x65, 0x4d, 0x6f, 0x76, 0x65, 0x52, 0x65, 0x71,
	0x75, 0x65, 0x73, 0x74, 0x1a, 0x17, 0x2e, 0x63, 0x68, 0x65, 0x73, 0x73, 0x2e, 0x4d, 0x61, 0x6b,
	0x65, 0x4d, 0x6f, 0x76, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x3d, 0x0a,
	0x08, 0x4a, 0x6f, 0x69, 0x6e, 0x47, 0x61, 0x6d, 0x65, 0x12, 0x16, 0x2e, 0x63, 0x68, 0x65, 0x73,
	0x73, 0x2e, 0x4a, 0x6f, 0x69, 0x6e, 0x47, 0x61, 0x6d, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73,
	0x74, 0x1a, 0x17, 0x2e, 0x63, 0x68, 0x65, 0x73, 0x73, 0x2e, 0x4a, 0x6f, 0x69, 0x6e, 0x47, 0x61,
	0x6d, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x30, 0x01, 0x42, 0x2c, 0x5a, 0x2a,
	0x67, 0x69, 0x74, 0x68, 0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x74, 0x6f, 0x78, 0x65, 0x65,
	0x65, 0x63, 0x2f, 0x63, 0x68, 0x65, 0x73, 0x73, 0x2f, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65,
	0x73, 0x2f, 0x67, 0x6f, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74,
	0x6f, 0x33,
}

var (
	file_chess_proto_rawDescOnce sync.Once
	file_chess_proto_rawDescData = file_chess_proto_rawDesc
)

func file_chess_proto_rawDescGZIP() []byte {
	file_chess_proto_rawDescOnce.Do(func() {
		file_chess_proto_rawDescData = protoimpl.X.CompressGZIP(file_chess_proto_rawDescData)
	})
	return file_chess_proto_rawDescData
}

var file_chess_proto_msgTypes = make([]protoimpl.MessageInfo, 4)
var file_chess_proto_goTypes = []interface{}{
	(*MakeMoveRequest)(nil),  // 0: chess.MakeMoveRequest
	(*MakeMoveResponse)(nil), // 1: chess.MakeMoveResponse
	(*JoinGameRequest)(nil),  // 2: chess.JoinGameRequest
	(*JoinGameResponse)(nil), // 3: chess.JoinGameResponse
}
var file_chess_proto_depIdxs = []int32{
	0, // 0: chess.Chess.MakeMove:input_type -> chess.MakeMoveRequest
	2, // 1: chess.Chess.JoinGame:input_type -> chess.JoinGameRequest
	1, // 2: chess.Chess.MakeMove:output_type -> chess.MakeMoveResponse
	3, // 3: chess.Chess.JoinGame:output_type -> chess.JoinGameResponse
	2, // [2:4] is the sub-list for method output_type
	0, // [0:2] is the sub-list for method input_type
	0, // [0:0] is the sub-list for extension type_name
	0, // [0:0] is the sub-list for extension extendee
	0, // [0:0] is the sub-list for field type_name
}

func init() { file_chess_proto_init() }
func file_chess_proto_init() {
	if File_chess_proto != nil {
		return
	}
	if !protoimpl.UnsafeEnabled {
		file_chess_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*MakeMoveRequest); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_chess_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*MakeMoveResponse); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_chess_proto_msgTypes[2].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*JoinGameRequest); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_chess_proto_msgTypes[3].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*JoinGameResponse); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	file_chess_proto_msgTypes[3].OneofWrappers = []interface{}{}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_chess_proto_rawDesc,
			NumEnums:      0,
			NumMessages:   4,
			NumExtensions: 0,
			NumServices:   1,
		},
		GoTypes:           file_chess_proto_goTypes,
		DependencyIndexes: file_chess_proto_depIdxs,
		MessageInfos:      file_chess_proto_msgTypes,
	}.Build()
	File_chess_proto = out.File
	file_chess_proto_rawDesc = nil
	file_chess_proto_goTypes = nil
	file_chess_proto_depIdxs = nil
}
