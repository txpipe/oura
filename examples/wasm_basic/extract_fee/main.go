package main

import (
	"github.com/extism/go-pdk"
)

//export map_u5c_tx
func map_u5c_tx() int32 {
	// unmarshal the U5C Tx data provided by the host
	var param map[string]interface{}
	err := pdk.InputJSON(&param)

	if err != nil {
		pdk.SetError(err)
		return 1
	}

	//pdk.Log(pdk.LogInfo, fmt.Sprintf("%v", param))

	// Here is where you get to do something interesting with the data. In this example, we just extract the fee data from the Tx
	fee := param["fee"].(interface{})

	// Use this method to return the mapped value back to the Oura pipeline.
	err = pdk.OutputJSON(fee)

	if err != nil {
		pdk.SetError(err)
		return 1
	}

	// return 0 for a successful operation and 1 for failure.
	return 0
}

func main() {}
