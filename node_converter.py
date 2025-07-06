

# load in the json file
import json
import os
def load_json_file(file_path):
    if not os.path.exists(file_path):
        raise FileNotFoundError(f"File {file_path} does not exist.")
    with open(file_path, 'r') as file:
        return json.load(file)
    
# save the json file
def save_json_file(file_path, data):
    with open(file_path, 'w') as file:
        json.dump(data, file, indent=4)

def camel_to_snake(input: str) -> str:
    out = ""
    for i, c in enumerate(input):
        if c.isupper() and i > 0:
            out += "_" + c.lower()
        else:
            out += c.lower()
    return out

import ast
import operator as op

# supported operators
_OPERATORS = {
    ast.Add: op.add,
    ast.Sub: op.sub,
    ast.Mult: op.mul,
    ast.Div: op.truediv,
    ast.Pow: op.pow,
    ast.USub: op.neg,
}

def eval_expr(expr: str):
    """
    Evaluate a math expression (numbers, + - * / **, and unary -) safely.
    """
    node = ast.parse(expr, mode='eval').body
    return _eval_node(node)

def _eval_node(node):
    if isinstance(node, ast.Num):  # <number>
        return node.n
    if isinstance(node, ast.BinOp):  # <left> <operator> <right>
        op_func = _OPERATORS[type(node.op)]
        return op_func(_eval_node(node.left), _eval_node(node.right))
    if isinstance(node, ast.UnaryOp):  # - <operand>
        op_func = _OPERATORS[type(node.op)]
        return op_func(_eval_node(node.operand))
    raise TypeError(f"Unsupported expression: {ast.dump(node)}")


def convert_io_type(ioType: str) -> str:
    """
    Convert the input/output type from the input format to the output format.
    """
    if ioType == "U8":
        return "uint8"
    elif ioType == "U16":
        return "uint16"
    elif ioType == "U32":
        return "uint32"
    elif ioType == "I8":
        return "int8"
    elif ioType == "I16":
        return "int16"
    elif ioType == "I32":
        return "int32"
    elif ioType == "F32":
        return "float"
    elif ioType == "F64":
        return "double"
    elif ioType == "Bool":
        return "bool"
    else:
        raise ValueError(f"Unknown IO type: {ioType}")

def convert_node(nodeType: str, data: dict, id: str) -> dict:
    """
    Convert a node from the input format to the output format.
    """
    outNode = {}
    pinData = {
        "in": {},
        "out": {}
    }
    key = ""
    
    if nodeType == "Constant":
        outNode["type"] = "constant"
        t = convert_io_type(data["itype"])
        outNode["config"] = {}
        outNode["config"]["type"] = t
        if t == "bool":
            outNode["config"]["value"] = bool(data["value"])
        else:
            outNode["config"]["value"] = float(eval_expr(data["value"]))
        
        key = data['node_name']
        pinData["out"]["0"] = "output"

    elif nodeType == "ApiInput":
        outNode["type"] = "api_input"
        config = {}
        config = {
            "type": convert_io_type(data["itype"])
            }
        if "min" in data:
            config["min"] = data["min"]
        if "max" in data:
            config["max"] = data["max"]
        if "default" in data:
            config["default"] = data["default"]
        if "timeout" in data:
            config["timeout"] = data["timeout"]

        outNode["config"] = {}
        outNode["config"]["config"] = config
        
        key = data['node_name']
        pinData["out"]["0"] = "output"

    elif nodeType == "ApiOutput":
        outNode["type"] = "api_output"
        outNode["config"] = {
            "type": convert_io_type(data["itype"])
        }
        
        key = data['node_name']
        pinData["in"]["0"] = "input"

    elif nodeType == "PIController":
        outNode["type"] = "pi_controller"
        outNode["config"] = {
            "Kp" : data["p"],
            "Ki" : data["i"],
            "i_limit" : data["i_limit"],
            "output_min" : data["output_min"],
            "output_max" : data["output_max"],
        }

        key = data['node_name']
        pinData["in"]["0"] = "input"
        pinData["in"]["1"] = "reset"
        pinData["out"]["0"] = "output"
        pinData["out"]["1"] = "saturation"
    
    elif nodeType == "VelEstimator":
        outNode["type"] = "vel_estimator"
        outNode["config"] = {
            "alpha": data["alpha"]
        }

        key = f"vel_estimator_{id}"
        pinData["in"]["0"] = "input"
        pinData["out"]["0"] = "output"

    elif nodeType == "LogicGate":
        outNode["type"] = "logic_gate"
        # get the first item in the “gType” dict
        gateType = data["gtype"]
        inCount = 1
        if(gateType != "NOT"):
            gateType, inCount = next(iter(data["gtype"].items()))
        gateType = gateType.lower()
        inCount = int(inCount)
        outNode["config"] = {
            "gate_type": gateType,
            "input_count": inCount
        }
        key = f"logic_gate_{id}"
        for i in range(inCount):
            pinData["in"][str(i)] = f"input_{i}"
        pinData["out"]["0"] = "output"

    elif nodeType == "Comparator":
        outNode["type"] = "comparator"
        compType = camel_to_snake(data["comparison"])
        outNode["config"] = {
            "type": convert_io_type(data["itype"]),
            "comparator_type": compType,
        }
        key = f"comparator_{id}"
        pinData["in"]["0"] = "input_0"
        pinData["in"]["1"] = "input_1"
        pinData["out"]["0"] = "output"
    
    elif nodeType == "MathOperation":
        outNode["type"] = "math_operation"
        outNode["config"] = {
            "type": convert_io_type(data["itype"]),
        }
        key = f"math_operation_{id}"
        if "Nary" in data["operator"]:
            outNode["config"]["op_type"] = "nary"
            inCount = int(data["operator"]["Nary"][1])
            op = data["operator"]["Nary"][0].lower()
            outNode["config"]["op"] = op
            outNode["config"]["input_count"] = inCount
            for i in range(inCount):
                pinData["in"][str(i)] = f"input_{i}"
        elif "BinaryOperation" in data["operator"]:
            outNode["config"]["op_type"] = "binary"
            op = data["operator"]["BinaryOperation"].lower()
            outNode["config"]["op"] = op
            pinData["in"]["0"] = "input_0"
            pinData["in"]["1"] = "input_1"
        elif "UnaryOperation" in data["operator"]:
            outNode["config"]["op_type"] = "unary"
            op = data["operator"]["UnaryOperation"].lower()
            outNode["config"]["op"] = op
            pinData["in"]["0"] = "input_0"
        
        pinData["out"]["0"] = "output"

    elif nodeType == "Multiplexer":
        outNode["type"] = "multiplexer"
        bits = int(data["input_bits"])
        outNode["config"] = {
            "input_bits": bits,
            "type": convert_io_type(data["itype"]),
        }
        key = f"multiplexer_{id}"
        for i in range(bits):
            pinData["in"][str(i)] = f"select_bit_{i}"
        for i in range(2**bits):
            pinData["in"][str(i + bits)] = f"input_{i}"
        pinData["out"]["0"] = "output"

    elif nodeType == "BitwiseSplit":
        outNode["type"] = "bitwise_split"
        bits = 0
        if data["num_bits"] <= 8:
            itype = "U8"
            bits = 8
        elif data["num_bits"] <= 16:
            itype = "U16"
            bits = 16
        elif data["num_bits"] <= 32:
            itype = "U32"
            bits = 32
        else:
            raise ValueError(f"Unknown bitwise split data type: {data['itype']}")
        
        outNode["config"] = {
            "type": convert_io_type(itype),
        }
        key = f"bitwise_split_{id}"
        pinData["in"]["0"] = "input"
        for i in range(bits):
            pinData["out"][str(i)] = f"output_bit_{i}"

    elif nodeType == "BitwiseJoin":
        outNode["type"] = "bitwise_join"
        bits = 0
        if data["num_bits"] <= 8:
            itype = "U8"
            bits = 8
        elif data["num_bits"] <= 16:
            itype = "U16"
            bits = 16
        elif data["num_bits"] <= 32:
            itype = "U32"
            bits = 32
        else:
            raise ValueError(f"Unknown bitwise join data type: {data['itype']}")
        
        outNode["config"] = {
            "type": convert_io_type(itype),
        }
        key = f"bitwise_join_{id}"
        pinData["out"]["0"] = "output"
        for i in range(bits):
            pinData["in"][str(i)] = f"input_bit_{i}"

    elif nodeType == "EdgeDelay":
        outNode["type"] = "edge_delay"
        outNode["config"] = {
            "rising_edge": data["rising_edge"],
            "falling_edge": not data["rising_edge"],
            "cycles": data["cycles"]
        }
        key = f"edge_delay_{id}"
        pinData["in"]["0"] = "input"
        pinData["out"]["0"] = "output"

    elif nodeType == "EdgeDetect":
        outNode["type"] = "edge_detect"
        outNode["config"] = {
            "rising_edge": data["rising_edge"],
            "falling_edge": not data["rising_edge"]
        }
        key = f"edge_detect_{id}"
        pinData["in"]["0"] = "input"
        pinData["out"]["0"] = "output"

    elif nodeType == "CycleDelay":
        outNode["type"] = "cycle_delay"
        outNode["config"] = {
            "cycles": data["cycles"],
            "type": convert_io_type(data["itype"])
        }
        key = f"cycle_delay_{id}"
        pinData["in"]["0"] = "input"
        pinData["out"]["0"] = "output"

    elif nodeType == "Converter":
        outNode["type"] = "converter"
        minMax = []
        minMax.append(data["input_min"])
        minMax.append(data["input_max"])
        minMax.append(data["output_min"])
        minMax.append(data["output_max"])
        for i in range(4):
            if minMax[i] == "":
                minMax[i] = 0.0
            else:
                try:
                    minMax[i] = float(minMax[i])
                except ValueError:
                    raise ValueError(f"Invalid min/max value: {minMax[i]}")
        directMode = data["direct_mode"]
        outNode["config"] = {
            "input_type": convert_io_type(data["input_type"]),
            "output_type": convert_io_type(data["output_type"]),
            "mode": "direct",
            "invert": bool(data["invert"])
        }
        if not directMode:
            outNode["config"]["input_min"] = minMax[0]
            outNode["config"]["input_max"] = minMax[1]
            outNode["config"]["output_min"] = minMax[2]
            outNode["config"]["output_max"] = minMax[3]
            outNode["config"]["mode"] = "scale"
        
        key = f"converter_{id}"
        pinData["in"]["0"] = "input"
        pinData["out"]["0"] = "output"

    elif nodeType == "Print":
        outNode["type"] = "print"
        outNode["config"] = {
            "name": data["name"],
            "type": convert_io_type(data["itype"]),
        }
        key = f"print_{id}"
        pinData["in"]["0"] = "input"
        pinData["in"]["1"] = "enable"

    elif nodeType == "Kins":

        # manual setting overrides

        dhparams = []
        dhparams.append({
            "alpha": -90.0,
            "a": 0.150,
            "d": 0.0,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": 0.0,
            "a": 0.570,
            "d": 0.0,
            "theta": -90.0
        })
        dhparams.append({
            "alpha": -90.0,
            "a": 0.130,
            "d": 0.0,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": 90.0,
            "a": 0.0,
            "d": 0.640,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": -90.0,
            "a": 0.0,
            "d": 0.0,
            "theta": 90.0
        })
        dhparams.append({
            "alpha": 0.0,
            "a": 0.0,
            "d": 0.095,
            "theta": 0.0
        })
        
        limits = {}
        limits["0"] = {
            "min_pos": -90,
            "max_pos": 90,
            "max_vel": 140,
            "max_acc": 100
        }
        limits["1"] = {
            "min_pos": -90,
            "max_pos": 90,
            "max_vel": 160,
            "max_acc": 100
        }
        limits["2"] = {
            "min_pos": -78,
            "max_pos": 90,
            "max_vel": 170,
            "max_acc": 100
        }
        limits["3"] = {
            "min_pos": 0,
            "max_pos": 0,
            "max_vel": 335,
            "max_acc": 100
        }
        limits["4"] = {
            "min_pos": -45,
            "max_pos": 225,
            "max_vel": 335,
            "max_acc": 100
        }
        limits["5"] = {
            "min_pos": 0,
            "max_pos": 0,
            "max_vel": 500,
            "max_acc": 100
        }

        cartesian_limits = {}
        cartesian_limits["x"] = {
            "min_pos": -1000,
            "max_pos": 1000,
            "max_vel": 1,   # m/s
            "max_acc": 10   # m/s^2
        }
        cartesian_limits["y"] = {
            "min_pos": -1000,
            "max_pos": 1000,
            "max_vel": 1,
            "max_acc": 10
        }
        cartesian_limits["z"] = {
            "min_pos": -1000,
            "max_pos": 1000,
            "max_vel": 1,
            "max_acc": 10
        }
        cartesian_limits["xangle"] = {
            "min_pos": -180,
            "max_pos": 180,
            "max_vel": 300,  # degrees/s
            "max_acc": 100   # degrees/s^2
        }
        cartesian_limits["yangle"] = {
            "min_pos": -180,
            "max_pos": 180,
            "max_vel": 300,
            "max_acc": 100
        }
        cartesian_limits["zangle"] = {
            "min_pos": -180,
            "max_pos": 180,
            "max_vel": 300,
            "max_acc": 100
        }

        tools = []
        # tools.append({  # tool 1, marker
        #     "position": [-0.1, 0.0, 0.037],
        #     "orientation": [0.0, 0.0, 0.0]
        # })
        tools.append({  # tool 1, marker offset to match mouse pen tip
            "position": [0.02, 0.0, 0.037],
            "orientation": [0.0, 0.0, 0.0]
        })

        tools.append({  # tool 2, stamp
            "position": [0.0, 0.0, 0.1],
            "orientation": [0.0, 0.0, 0.0]
        })

        tools.append({  # tool 3, foam block holder
            "position": [0.02, 0.0, 0.046],
            "orientation": [0.0, 0.0, 0.0]
        })

        outNode["type"] = "kins"
        outNode["config"] = {
            "dh_params": dhparams,
            "limits": limits,
            "max_jog_speed": 0.1,
            "cartesian_limits": cartesian_limits,
            "tools": tools,
        }
        key = data["name"]
        pinData["in"]["0"] = "j1_fbk_pos"
        pinData["in"]["1"] = "j2_fbk_pos"
        pinData["in"]["2"] = "j3_fbk_pos"
        pinData["in"]["3"] = "j4_fbk_pos"
        pinData["in"]["4"] = "j5_fbk_pos"
        pinData["in"]["5"] = "j6_fbk_pos"
        pinData["in"]["6"] = "j1_cmd_pos"
        pinData["in"]["7"] = "j2_cmd_pos"
        pinData["in"]["8"] = "j3_cmd_pos"
        pinData["in"]["9"] = "j4_cmd_pos"
        pinData["in"]["10"] = "j5_cmd_pos"
        pinData["in"]["11"] = "j6_cmd_pos"
        pinData["in"]["12"] = "x_cmd_pos"
        pinData["in"]["13"] = "y_cmd_pos"
        pinData["in"]["14"] = "z_cmd_pos"
        pinData["in"]["15"] = "xangle_cmd_pos"
        pinData["in"]["16"] = "yangle_cmd_pos"
        pinData["in"]["17"] = "zangle_cmd_pos"
        pinData["in"]["18"] = "jog_axis_select"
        pinData["in"]["19"] = "jog_vel"
        pinData["in"]["20"] = "jog_mode"
        pinData["in"]["21"] = "control_mode"
        pinData["in"]["22"] = "speed_override"
        pinData["in"]["23"] = "tool_select"
        pinData["in"]["24"] = "reset"

        pinData["out"]["0"] = "j1_cmd_pos"
        pinData["out"]["1"] = "j2_cmd_pos"
        pinData["out"]["2"] = "j3_cmd_pos"
        pinData["out"]["3"] = "j4_cmd_pos"
        pinData["out"]["4"] = "j5_cmd_pos"
        pinData["out"]["5"] = "j6_cmd_pos"
        pinData["out"]["6"] = "x_fbk_pos"
        pinData["out"]["7"] = "y_fbk_pos"
        pinData["out"]["8"] = "z_fbk_pos"
        pinData["out"]["9"] = "xangle_fbk_pos"
        pinData["out"]["10"] = "yangle_fbk_pos"
        pinData["out"]["11"] = "zangle_fbk_pos"

    elif nodeType == "Mouse":

        # manual setting overrides

        dhparams = []
        dhparams.append({
            "alpha": -90.0,
            "a": 0.0,
            "d": 0.0,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": 0.0,
            "a": 0.210,
            "d": 0.0,
            "theta": -90.0
        })
        dhparams.append({
            "alpha": -90.0,
            "a": 0.0,
            "d": 0.0,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": 90.0,
            "a": 0.0,
            "d": 0.210,
            "theta": 0.0
        })
        dhparams.append({
            "alpha": -90.0,
            "a": 0.0,
            "d": 0.0,
            "theta": 90.0
        })
        dhparams.append({
            "alpha": 0.0,
            "a": 0.0,
            "d": 0.0,
            "theta": 0.0
        })

        GEAR_MOTOR_COUNT_PER_REV = 57.0/13.0 * 1000.0 * 4.0
        
        joint_params = []
        joint_params.append({
                "count_per_rev": GEAR_MOTOR_COUNT_PER_REV * 14,
                "home_pos": 53000
            })
        joint_params.append({
                "count_per_rev": GEAR_MOTOR_COUNT_PER_REV * 11.56,
                "home_pos": -42000
            })
        joint_params.append({
                "count_per_rev": GEAR_MOTOR_COUNT_PER_REV * 11.56,
                "home_pos": -56300
            })
        joint_params.append({
                "count_per_rev": 2048,
                "home_pos": -49
            })
        joint_params.append({
                "count_per_rev": -2048,
                "home_pos": -57
            })
        joint_params.append({
                "count_per_rev": -2048,
                "home_pos": 1024
            })


        outNode["type"] = "mouse"
        outNode["config"] = {
            "dh_params": dhparams,
            "joint_params": joint_params
        }
        key = data["name"]
        pinData["in"]["0"] = "j1_fbk_pos"
        pinData["in"]["1"] = "j2_fbk_pos"
        pinData["in"]["2"] = "j3_fbk_pos"
        pinData["in"]["3"] = "j4_fbk_pos"
        pinData["in"]["4"] = "j5_fbk_pos"
        pinData["in"]["5"] = "j6_fbk_pos"

        pinData["out"]["0"] = "x_fbk_pos"
        pinData["out"]["1"] = "y_fbk_pos"
        pinData["out"]["2"] = "z_fbk_pos"
        pinData["out"]["3"] = "xangle_fbk_pos"
        pinData["out"]["4"] = "yangle_fbk_pos"
        pinData["out"]["5"] = "zangle_fbk_pos"

    elif nodeType == "SerialDevice":
        outNode["type"] = "em_serial_device"
        pinData["in"]["0"] = "em_device"
        outNode["config"] = {
            "enabled": data["enabled"],
            "comm": {
                "device_address": data["addr"],
                "timeout_tries": data["timeout"],
            },
            "device": {
                "device_descriptor": data["descriptor"],
                "cyclic_write_regs": [],
                "cyclic_read_regs": [],
                "async_write_regs": [],
                "async_read_regs": [],
                "set": []
            }
        }
        key = data['node_name']

        # pin data is unknown for raw serial devices
        # it depends on the connected serial read/write nodes

    elif nodeType == "SerialRead":
        if "AsyncReg" in data["dev"]:
            outNode["async_read_regs"] = {
                "name": data["name"],
                "update_rate_cycles": data["dev"]["AsyncReg"]["update_cycles"]
            }
        else:
            outNode["cyclic_read_regs"] = {
                "name": data["name"],
                "cyclic_index": data["dev"]["CyclicReg"]["cyclic_index"],
                "sync_with_node": data["dev"]["CyclicReg"]["sync_node"]
            }

    elif nodeType == "SerialWrite":
        if "AsyncReg" in data["dev"]:
            outNode["async_write_regs"] = {
                "name": data["name"],
                "update_rate_cycles": data["dev"]["AsyncReg"]["update_cycles"]
            }
        else:
            outNode["cyclic_write_regs"] = {
                "name": data["name"],
                "cyclic_index": data["dev"]["CyclicReg"]["cyclic_index"],
                "sync_with_node": data["dev"]["CyclicReg"]["sync_node"]
            }

    elif nodeType == "SerialSet":
        outNode["set"] = {
            "name": data["name"],
            "value": float(data["value"])
        }

    elif nodeType == "GetGlobalVariable":
        outNode["type"] = "get_global_variable"
        pinData["out"]["0"] = "output"
        outNode["config"] = {
            "variables": {
                "output": data['name']
            }
        }
        key = f"get_global_variable_{id}"

    elif nodeType == "SetGlobalVariable":
        outNode["type"] = "set_global_variable"
        pinData["in"]["0"] = "input"
        pinData["in"]["1"] = "input_enable"
        outNode["config"] = {
            "variables": {
                "input": data['name']
            }
        }
        key = f"set_global_variable_{id}"

    else:
        raise ValueError(f"Unknown node type: {nodeType}")

    return (key, outNode), pinData

if __name__ == "__main__":
    
    inConfig = load_json_file(os.path.expanduser("~/test.json"))

    outConfig = {}

    outConfig["info"] = inConfig["info"]    # info section is the same
    outConfig["networks"] = {}               # networks will be filled later

    # create user global variables
    outConfig["user_node_vars"] = {}
    for var in inConfig["set_node_vars"]:
        outConfig["user_node_vars"][var["name"]] = {
            "type": convert_io_type(var["pin_type"])
        }


   # set global variables
    outConfig["node_var_values"] = {}  # not supported yet 


    for network in inConfig["networks"]:

        newNetwork = {}
        serialDeviceConfigs = {
            "writes": {},
            "reads": {},
            "sets": {}
        }

        pinData = {}

        # copy the network info
        newNetwork["enable"] = network["enabled"]
        newNetwork["dynamic_enable_starting"] = network["dynamic_enable_starting"]
        newNetwork["type"] = network["net_type"].lower()
        newNetwork["timeout_usec"] = network["timeout"]
        newNetwork["update_cycle_trigger_count"] = network["update_cycle_trigger_count"]
        newNetwork["execution_order"] = network["execution_index"]


        # copy the nodes
        nodes = {}
        for nodeID, node in network["nodes"]["nodes"].items():
            nodeData = node["value"]

            # get the first key and its node data
            nodetype, first_entry = next(iter(nodeData.items()))
            nodeData = nodeData[nodetype]

            (key, outNode), p = convert_node(nodetype, nodeData, nodeID)

            if nodetype == "SerialWrite" or nodetype == "SerialRead" or nodetype == "SerialSet":
                # these are not actual nodes, but simplify the serial device config
                # they need to be added to the serial device config afterwards
                
                if nodetype == "SerialWrite":
                    if nodeID in serialDeviceConfigs["writes"]:
                        raise ValueError(f"Serial write node {nodeID} already exists")
                    serialDeviceConfigs["writes"][nodeID] = outNode
                elif nodetype == "SerialRead":
                    if nodeID in serialDeviceConfigs["reads"]:
                        raise ValueError(f"Serial read node {nodeID} already exists")
                    serialDeviceConfigs["reads"][nodeID] = outNode
                elif nodetype == "SerialSet":
                    if nodeID in serialDeviceConfigs["sets"]:
                        raise ValueError(f"Serial set node {nodeID} already exists")
                    serialDeviceConfigs["sets"][nodeID] = outNode
                continue


            if key in nodes:
                raise ValueError(f"Node {key} already exists in network {network['name']}")

            nodes[key] = outNode
            p["name"] = key
            p["type"] = nodetype
            pinData[nodeID] = p

        
        # add the serial device configs
        removeKeys = []
        for i, wire in enumerate(network["nodes"]["wires"]):
            src = str(wire["out_pin"]["node"])
            dst = str(wire["in_pin"]["node"])

            if src in serialDeviceConfigs["writes"]:
                # write node leading to a serial device
                deviceName = pinData[dst]["name"]
                write = serialDeviceConfigs["writes"][src]
                if "device_name" not in serialDeviceConfigs["writes"][src]:
                    serialDeviceConfigs["writes"][src]["device_name"] = []
                serialDeviceConfigs["writes"][src]["device_name"].append(deviceName)
                for k, v in write.items():
                    if k == "device_name":
                        continue
                    nodes[deviceName]["config"]["device"][k].append(v)
                    
                removeKeys.append(i)
            elif dst in serialDeviceConfigs["reads"]:
                # read node leading from a serial device
                deviceName = pinData[src]["name"]
                read = serialDeviceConfigs["reads"][dst]
                if "device_name" not in serialDeviceConfigs["reads"][dst]:
                    serialDeviceConfigs["reads"][dst]["device_name"] = []
                serialDeviceConfigs["reads"][dst]["device_name"].append(deviceName)
                for k, v in read.items():
                    if k == "device_name":
                        continue
                    nodes[deviceName]["config"]["device"][k].append(v)
                removeKeys.append(i)
            elif src in serialDeviceConfigs["sets"]:
                deviceName = pinData[dst]["name"]
                set = serialDeviceConfigs["sets"][src]
                if "device_name" not in serialDeviceConfigs["sets"][src]:
                    serialDeviceConfigs["sets"][src]["device_name"] = []
                serialDeviceConfigs["sets"][src]["device_name"].append(deviceName)
                for k, v in set.items():
                    if k == "device_name":
                        continue
                    nodes[deviceName]["config"]["device"][k].append(v)
                removeKeys.append(i)

        # remove the wires that were used for serial read/write nodes
        for i in sorted(removeKeys, reverse=True):
            network["nodes"]["wires"].pop(i)

        
        # make the remaining valid connections
        i = 0
        connections = {}
        for wire in network["nodes"]["wires"]:
            src = str(wire["out_pin"]["node"])
            dst = str(wire["in_pin"]["node"])

            done = False
            device_index = 0

            while not done:

                src_node = None
                dst_node = None

                src_pin = None
                dst_pin = None

                serial_node = False

                if src in serialDeviceConfigs["reads"]:
                    src_node = serialDeviceConfigs["reads"][src]["device_name"][device_index] # use the serial device name
                    r = next(iter(serialDeviceConfigs["reads"][src].values()))
                    src_pin = r["name"]
                    dst_pin = pinData[dst]["in"].get(str(wire["in_pin"]["input"]), None)
                    if len(serialDeviceConfigs["reads"][src]["device_name"]) <= device_index + 1:
                        done = True
                    serial_node = True

                if dst in serialDeviceConfigs["writes"]:
                    dst_node = serialDeviceConfigs["writes"][dst]["device_name"][device_index] # use the serial device name
                    w = next(iter(serialDeviceConfigs["writes"][dst].values()))
                    dst_pin = w["name"]
                    src_pin = pinData[src]["out"].get(str(wire["out_pin"]["output"]), None)
                    if len(serialDeviceConfigs["writes"][dst]["device_name"]) <= device_index + 1:
                        done = True
                    serial_node = True

                if src_node is None:
                    src_node = pinData[src]["name"]
                if dst_node is None:
                    dst_node = pinData[dst]["name"]

                
                if src_pin is None and dst_pin is None:
                    src_pin = pinData[src]["out"].get(str(wire["out_pin"]["output"]), None)
                    dst_pin = pinData[dst]["in"].get(str(wire["in_pin"]["input"]), None)


                
                if src_pin is None or dst_pin is None:
                    raise ValueError(f"Invalid pin connection from {src} to {dst}")
                
                connections[i] = {
                    "src_node": src_node,
                    "src_port": src_pin,
                    "dst_node": dst_node,
                    "dst_port": dst_pin
                }
                i += 1
                device_index += 1
                if not serial_node:
                    done = True
        

        newNetwork["connections"] = connections
        newNetwork["nodes"] = nodes

        # add the network to the output config
        if(outConfig["networks"].get(network["name"]) is not None):
            raise ValueError(f"Network {network['name']} already exists")
        outConfig["networks"][network["name"]] = newNetwork

    print(outConfig)

    # save the output config
    save_json_file(os.path.expanduser("~/github/controller-software/controller-software/core/zynq_files/controller/config/user/new_node_config.json"), outConfig)