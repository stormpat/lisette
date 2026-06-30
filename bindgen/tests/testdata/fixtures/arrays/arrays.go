package arrays

// Go fixed-size arrays [N]T map faithfully to Lisette Array<T, N>, with the
// length kept in the type. This fixture covers every position bindgen handles.

// Return position: [32]byte becomes Array<byte, 32>.
func Sum() [32]byte { return [32]byte{} }

// Parameter position: [4]byte becomes Array<byte, 4> (previously skipped).
func Take(addr [4]byte) {}

// Nested array: [2][3]int becomes Array<Array<int, 3>, 2>.
func Grid() [2][3]int { return [2][3]int{} }

// Slice of array: [][4]int becomes Slice<Array<int, 4>>.
func Rows() [][4]int { return nil }

// Pointer to array: *[8]byte becomes Ref<Array<byte, 8>>.
func Ptr() *[8]byte { return nil }

// Array as a map key (arrays are comparable when the element is).
func TakesArrayKey(m map[[2]uint16]int) {}

// Alias to an array peels to the array type.
type Digest = [16]byte

func Compute() Digest { return Digest{} }

// Struct with an array field, a map-value array field, and an array-returning
// method (the field and map-value positions were previously skipped).
type Holder struct {
	Data  [3]int
	Pairs map[string][4]byte
}

func (h Holder) First() [3]int { return h.Data }
