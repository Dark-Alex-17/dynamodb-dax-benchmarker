package utils

func MapValues[K comparable, V any](inputMap map[K]V) []V {
	valuesSlice := make([]V, 0)

	for _, value := range inputMap {
		valuesSlice = append(valuesSlice, value)
	}

	return valuesSlice
}
