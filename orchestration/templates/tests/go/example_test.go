// Example Go Test Template
//
// Replace this with actual tests for your component.

package main

import (
	"testing"
)

func TestExample(t *testing.T) {
	// Example test - replace with actual tests
	if false {
		t.Error("Example test failed")
	}
}

func TestStringOperations(t *testing.T) {
	result := "hello" + " " + "world"
	expected := "hello world"

	if result != expected {
		t.Errorf("Expected '%s', got '%s'", expected, result)
	}

	if len(result) != 11 {
		t.Errorf("Expected length 11, got %d", len(result))
	}
}

func TestSliceOperations(t *testing.T) {
	items := []int{1, 2, 3}
	items = append(items, 4)

	if len(items) != 4 {
		t.Errorf("Expected length 4, got %d", len(items))
	}

	if items[len(items)-1] != 4 {
		t.Errorf("Expected last element to be 4, got %d", items[len(items)-1])
	}
}

// TODO: Replace these examples with actual tests
// Example test structure:
//
// func TestFunctionName(t *testing.T) {
//     // Arrange
//     input := setupTestData()
//
//     // Act
//     result := functionUnderTest(input)
//
//     // Assert
//     if result != expectedValue {
//         t.Errorf("Expected %v, got %v", expectedValue, result)
//     }
// }
