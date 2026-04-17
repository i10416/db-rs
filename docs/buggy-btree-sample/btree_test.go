package btree

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func Test_Node_String(t *testing.T) {
	tests := []struct {
		name     string
		node     *Node[int, string]
		expected string
	}{
		{
			name: "leaf node with keys",
			node: &Node[int, string]{
				keys:   []int{1, 2, 3},
				values: []string{"a", "b", "c"},
				isLeaf: true,
			},
			expected: "[Leaf [1 2 3]]",
		},
		{
			name: "internal node with keys",
			node: &Node[int, string]{
				keys:   []int{10, 20},
				values: []string{"x", "y"},
				isLeaf: false,
			},
			expected: "[Internal [10 20]]",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.expected, tt.node.String())
		})
	}
}

func Test_NewBTree(t *testing.T) {
	tests := []struct {
		name     string
		order    int
		expected string
	}{
		{name: "order 3", order: 3, expected: "BTree(order=3) <empty>"},
		{name: "order 4", order: 4, expected: "BTree(order=4) <empty>"},
		{name: "order 5", order: 5, expected: "BTree(order=5) <empty>"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			bt := NewBTree[int, string](tt.order)
			assert.Equal(t, tt.expected, bt.String())
			assert.Equal(t, tt.order, bt.order)
			assert.NotNil(t, bt.root)
			assert.True(t, bt.root.isLeaf)
		})
	}
}

func Test_Search(t *testing.T) {
	t.Run("empty tree", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		value, found := bt.Search(10)
		assert.False(t, found)
		assert.Empty(t, value)
	})

	t.Run("single level tree", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		bt.root.keys = []int{10, 20, 30}
		bt.root.values = []string{"a", "b", "c"}

		tests := []struct {
			key       int
			wantValue string
			wantFound bool
		}{
			{key: 10, wantValue: "a", wantFound: true},
			{key: 20, wantValue: "b", wantFound: true},
			{key: 30, wantValue: "c", wantFound: true},
			{key: 25, wantValue: "", wantFound: false},
			{key: 5, wantValue: "", wantFound: false},
			{key: 35, wantValue: "", wantFound: false},
		}

		for _, tt := range tests {
			t.Run(fmt.Sprintf("key_%d", tt.key), func(t *testing.T) {
				value, found := bt.Search(tt.key)
				assert.Equal(t, tt.wantFound, found)
				assert.Equal(t, tt.wantValue, value)
			})
		}
	})

	t.Run("tree with float values", func(t *testing.T) {
		bt := NewBTree[int, float32](3)
		bt.root.keys = []int{10, 20, 30}
		bt.root.values = []float32{1.23, 3.45, 5.67}

		value, found := bt.Search(20)
		assert.True(t, found)
		assert.Equal(t, float32(3.45), value)
	})

	t.Run("multiple levels tree", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		bt.root.keys = []int{10, 20}
		bt.root.isLeaf = false
		bt.root.children = []*Node[int, string]{
			{keys: []int{1, 5, 9}, values: []string{"a", "b", "c"}, isLeaf: true, parent: bt.root},
			{keys: []int{15, 18, 19}, values: []string{"d", "e", "f"}, isLeaf: true, parent: bt.root},
			{keys: []int{25, 28, 30}, values: []string{"g", "h", "i"}, isLeaf: true, parent: bt.root},
		}

		tests := []struct {
			key       int
			wantValue string
			wantFound bool
		}{
			{key: 1, wantValue: "a", wantFound: true},
			{key: 18, wantValue: "e", wantFound: true},
			{key: 28, wantValue: "h", wantFound: true},
			{key: 22, wantValue: "", wantFound: false},
			{key: 100, wantValue: "", wantFound: false},
		}

		for _, tt := range tests {
			t.Run(fmt.Sprintf("key_%d", tt.key), func(t *testing.T) {
				value, found := bt.Search(tt.key)
				assert.Equal(t, tt.wantFound, found)
				assert.Equal(t, tt.wantValue, value)
			})
		}

		assert.Contains(t, bt.String(), "[Internal [10 20]]")
		assert.Contains(t, bt.String(), "[Leaf [25 28 30]]")
	})
}

func Test_Insert(t *testing.T) {
	t.Run("into empty root", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		err := bt.Insert(10, "a")
		assert.NoError(t, err)

		value, found := bt.Search(10)
		assert.True(t, found)
		assert.Equal(t, "a", value)
	})

	t.Run("duplicate key returns error", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		bt.root.keys = []int{20}
		bt.root.values = []string{"b"}

		err := bt.Insert(20, "new_b")
		if assert.Error(t, err) {
			assert.Contains(t, err.Error(), "already exists")
		}
	})

	t.Run("with sort needed", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		bt.root.keys = []int{10, 20}
		bt.root.values = []string{"a", "b"}

		err := bt.Insert(15, "b'")
		assert.NoError(t, err)

		value, found := bt.Search(15)
		assert.True(t, found)
		assert.Equal(t, "b'", value)
		assert.Equal(t, []int{10, 15, 20}, bt.root.keys)
	})

	t.Run("sequence with multiple keys", func(t *testing.T) {
		bt := NewBTree[int, string](3)
		insertions := []struct {
			key int
			val string
		}{
			{30, "c"}, {10, "a"}, {20, "b"}, {40, "d"},
		}

		for _, ins := range insertions {
			err := bt.Insert(ins.key, ins.val)
			assert.NoError(t, err)
		}

		for _, ins := range insertions {
			value, found := bt.Search(ins.key)
			assert.True(t, found, "key %d should be found", ins.key)
			assert.Equal(t, ins.val, value)
		}
	})
}

func Test_InsertAtIndex(t *testing.T) {
	tests := []struct {
		name     string
		arr      []int
		value    int
		idx      int
		expected []int
	}{
		{
			name:     "insert in middle",
			arr:      []int{1, 2, 3, 4, 5},
			value:    99,
			idx:      2,
			expected: []int{1, 2, 99, 3, 4, 5},
		},
		{
			name:     "insert at beginning",
			arr:      []int{2, 3, 4},
			value:    1,
			idx:      0,
			expected: []int{1, 2, 3, 4},
		},
		{
			name:     "insert at end",
			arr:      []int{1, 2, 3},
			value:    4,
			idx:      3,
			expected: []int{1, 2, 3, 4},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			original := make([]int, len(tt.arr))
			copy(original, tt.arr)

			result := insertAtIndex(tt.arr, tt.value, tt.idx)
			assert.Equal(t, tt.expected, result)
			assert.Equal(t, original, tt.arr[:len(original)])
		})
	}
}

func Test_RootSplit(t *testing.T) {
	bt := NewBTree[int, string](3)
	bt.root.keys = []int{10, 20, 30}
	bt.root.values = []string{"a", "b", "c"}

	assert.Equal(t, bt.order, len(bt.root.keys))
	assert.True(t, bt.root.isLeaf)

	err := bt.Insert(40, "d")
	assert.NoError(t, err)

	assert.Equal(t, 1, len(bt.root.keys))
	assert.Equal(t, 30, bt.root.keys[0])
	assert.False(t, bt.root.isLeaf)
	assert.Equal(t, 2, len(bt.root.children))
	assert.Equal(t, []int{10, 20}, bt.root.children[0].keys)
	assert.Equal(t, []int{40}, bt.root.children[1].keys)
	assert.True(t, bt.root.children[0].isLeaf)
	assert.True(t, bt.root.children[1].isLeaf)
}

func Test_Insert_Routing(t *testing.T) {
	bt := NewBTree[int, string](3)

	for _, kv := range []struct {
		k int
		v string
	}{{10, "a"}, {20, "b"}, {30, "c"}} {
		assert.NoError(t, bt.Insert(kv.k, kv.v))
	}

	tests := []struct {
		name        string
		key         int
		val         string
		expectChild int // 0=left, 1=right
	}{
		{name: "route to left child", key: 5, val: "e", expectChild: 0},
		{name: "route to right child", key: 40, val: "d", expectChild: 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := bt.Insert(tt.key, tt.val)
			assert.NoError(t, err)
			assert.Contains(t, bt.root.children[tt.expectChild].keys, tt.key)
		})
	}
}

func Test_Split_NonRootNode(t *testing.T) {
	bt := NewBTree[int, string](3)

	keys := []int{10, 20, 30, 40, 50, 60}
	for _, k := range keys {
		assert.NoError(t, bt.Insert(k, fmt.Sprintf("val_%d", k)))
	}

	err := bt.Insert(70, "val_70")
	assert.NoError(t, err)

	assert.Equal(t, []int{30, 60}, bt.root.keys)
	assert.Equal(t, 3, len(bt.root.children))
	assert.Equal(t, []int{10, 20}, bt.root.children[0].keys)
	assert.Equal(t, []int{40, 50}, bt.root.children[1].keys)
	assert.Equal(t, []int{70}, bt.root.children[2].keys)
}

func Test_FullInsertSequence(t *testing.T) {
	tests := []struct {
		name       string
		order      int
		insertions []struct {
			key int
			val string
		}
		expectedRoot []int
		numChildren  int
	}{
		{
			name:  "7 sequential insertions order 3",
			order: 3,
			insertions: []struct {
				key int
				val string
			}{
				{10, "a"}, {20, "b"}, {30, "c"}, {40, "d"},
				{50, "e"}, {60, "f"}, {70, "g"},
			},
			expectedRoot: []int{30, 60},
			numChildren:  3,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			bt := NewBTree[int, string](tt.order)

			for _, ins := range tt.insertions {
				err := bt.Insert(ins.key, ins.val)
				assert.NoError(t, err)
			}

			assert.Equal(t, tt.expectedRoot, bt.root.keys)
			assert.Equal(t, tt.numChildren, len(bt.root.children))

			for _, ins := range tt.insertions {
				val, found := bt.Search(ins.key)
				assert.True(t, found, "key %d not found", ins.key)
				assert.Equal(t, ins.val, val)
			}
		})
	}
}

func Test_Insert_DuplicateInMultiLevel(t *testing.T) {
	bt := NewBTree[int, string](3)

	for _, kv := range []struct {
		k int
		v string
	}{{10, "a"}, {20, "b"}, {30, "c"}, {40, "d"}} {
		assert.NoError(t, bt.Insert(kv.k, kv.v))
	}

	err := bt.Insert(10, "duplicate")
	if assert.Error(t, err) {
		assert.Contains(t, err.Error(), "already exists")
	}
}

func Test_Cascade(t *testing.T) {
	t.Run("all keys searchable after many inserts", func(t *testing.T) {
		bt := NewBTree[int, string](3)

		for i := 10; i <= 200; i += 10 {
			err := bt.Insert(i, fmt.Sprintf("val_%d", i))
			assert.NoError(t, err, "insert %d failed", i)
		}

		for i := 10; i <= 200; i += 10 {
			val, found := bt.Search(i)
			assert.True(t, found, "key %d not found", i)
			assert.Equal(t, fmt.Sprintf("val_%d", i), val)
		}

		verifyParentPointers(t, bt.root, nil)
	})

	t.Run("all nodes respect order constraint", func(t *testing.T) {
		bt := NewBTree[int, string](3)

		for i := 1; i <= 50; i++ {
			err := bt.Insert(i, fmt.Sprintf("val_%d", i))
			assert.NoError(t, err)
		}

		maxKeys := findMaxKeysInTree(bt.root)
		assert.LessOrEqual(t, maxKeys, 3, "Some node exceeds order - cascade failed!")

		for i := 1; i <= 50; i++ {
			_, found := bt.Search(i)
			assert.True(t, found)
		}
	})

	t.Run("root splits multiple times", func(t *testing.T) {
		bt := NewBTree[int, string](3)

		for i := 1; i <= 100; i++ {
			err := bt.Insert(i, fmt.Sprintf("val_%d", i))
			assert.NoError(t, err)
		}

		assert.False(t, bt.root.isLeaf, "root should be internal")
		depth := treeDepth(bt.root)
		assert.GreaterOrEqual(t, depth, 3, "with 100 keys, should have height ≥ 3")

		for i := 1; i <= 100; i++ {
			_, found := bt.Search(i)
			assert.True(t, found, "key %d not found", i)
		}
	})
}

func Test_Split_InternalNodeChildren(t *testing.T) {
	bt := NewBTree[int, string](3)

	keys := []int{25, 10, 40, 5, 30, 50, 15, 35, 20, 45}
	for _, k := range keys {
		err := bt.Insert(k, fmt.Sprintf("val_%d", k))
		assert.NoError(t, err, "insert %d failed", k)
	}

	for _, child := range bt.root.children {
		if !child.isLeaf {
			assert.Equal(t, len(child.keys)+1, len(child.children),
				"Internal node children count wrong")
		}
	}
}

func Test_Cascade_InternalNodeSplit(t *testing.T) {
	bt := NewBTree[int, string](3)

	keys := []int{
		10, 20, 30, 40,
		50, 60, 70,
		5, 15, 25, 35, 45, 55, 65, 75,
		12, 18, 22, 28,
	}

	for _, k := range keys {
		err := bt.Insert(k, fmt.Sprintf("val_%d", k))
		assert.NoError(t, err)
	}

	verifyInternalNodes(t, bt, bt.root)

	for _, k := range keys {
		_, found := bt.Search(k)
		assert.True(t, found, "key %d not found", k)
	}
}

func Test_Cascade_ParentPointersDeep(t *testing.T) {
	bt := NewBTree[int, string](3)

	for i := 1; i <= 200; i++ {
		err := bt.Insert(i, fmt.Sprintf("val_%d", i))
		assert.NoError(t, err, "insert %d failed", i)
	}

	verifyParentPointers(t, bt.root, nil)
}

func Test_Cascade_RandomStressTest(t *testing.T) {
	bt := NewBTree[int, string](3)

	keys := make([]int, 500)
	for i := 0; i < 500; i++ {
		keys[i] = i * 7 % 997 // Semi-random pattern
	}

	for _, k := range keys {
		err := bt.Insert(k, fmt.Sprintf("val_%d", k))
		assert.NoError(t, err)
	}

	verifyTreeProperties(t, bt, bt.root)

	for _, k := range keys {
		_, found := bt.Search(k)
		assert.True(t, found, "key %d not found", k)
	}
}

func findMaxKeysInTree[K Ordered, V any](n *Node[K, V]) int {
	if n == nil {
		return 0
	}

	maxKeys := len(n.keys)
	for _, child := range n.children {
		if childMax := findMaxKeysInTree(child); childMax > maxKeys {
			maxKeys = childMax
		}
	}
	return maxKeys
}

func treeDepth[K Ordered, V any](n *Node[K, V]) int {
	if n == nil || n.isLeaf {
		return 1
	}
	maxDepth := 0
	for _, child := range n.children {
		if depth := treeDepth(child); depth > maxDepth {
			maxDepth = depth
		}
	}
	return maxDepth + 1
}

func verifyParentPointers[K Ordered, V any](t *testing.T, n *Node[K, V], expectedParent *Node[K, V]) {
	if n == nil {
		return
	}
	assert.Equal(t, expectedParent, n.parent, "Node %v has wrong parent", n.keys)
	for _, child := range n.children {
		verifyParentPointers(t, child, n)
	}
}

func verifyInternalNodes[K Ordered, V any](t *testing.T, bt *BTree[K, V], n *Node[K, V]) {
	if n == nil {
		return
	}

	if !n.isLeaf {
		assert.LessOrEqual(t, len(n.keys), bt.order,
			"Internal node has %d keys > order %d", len(n.keys), bt.order)
		assert.Equal(t, len(n.keys)+1, len(n.children),
			"Internal node: %d keys but %d children", len(n.keys), len(n.children))
	}

	for _, child := range n.children {
		verifyInternalNodes(t, bt, child)
	}
}

func verifyTreeProperties[K Ordered, V any](t *testing.T, bt *BTree[K, V], n *Node[K, V]) {
	if n == nil {
		return
	}

	for i := 1; i < len(n.keys); i++ {
		assert.Less(t, n.keys[i-1], n.keys[i], "Keys not sorted in node")
	}

	assert.LessOrEqual(t, len(n.keys), bt.order, "Node has too many keys")

	if !n.isLeaf {
		assert.Equal(t, len(n.keys)+1, len(n.children), "Internal node children count wrong")
	}

	if n.isLeaf {
		assert.Empty(t, n.children, "Leaf should have no children")
	}

	for _, child := range n.children {
		verifyTreeProperties(t, bt, child)
	}
}
