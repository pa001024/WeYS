#Requires AutoHotkey v2.0
#Include lib.ahk

_auto_emo() {
    Click 756, 838
    Sleep 500
    click 478, 654
}

_auto_cook() {
    while WinActive("ahk_exe YuanShen.exe") {
        if not CheckColor(303, 50, "96D720") {
            _auto_f2()
            Sleep 7000
        }
        _auto_msg("你好！", false)
        Sleep 1500
        _auto_msg("可以让我的几个朋友进来打几个怪不~~ ", false)
        Sleep 4000
        _auto_msg("就是枫丹湖中垂柳右边的地方传奇，每天刷新的~~ 2分钟差不多打完了~", false)
        Sleep 4000
        _auto_emo()
        Sleep 1000
        reply := ""
        loop 30 {
            if PixelSearch(&x2, &y2, 345, 400, 503, 759, 0xFFFFFF) {
                WinGetClientPos(&x, &y, &w, &h)
                cap_and_ocr_text(x + x2, y + y2 + 40, 120, 30)
                Sleep 150
                reply := A_Clipboard
                break
            }
            Sleep 1000
        }
        if RegExMatch(reply, "可以|^行|OK|^好|哦|嗯") {
            _auto_msg("我先走啦 可以的话一会就同意一下~~ 谢谢啦~~")
            Sleep 1000
        } else {
            _auto_msg("不好意思打扰啦！")
            _auto_exit()
            Sleep 15000
            continue
        }
        if not CheckColor(303, 50, "96D720") {
            while CheckColor(300, 300, "1C1C22") or CheckColor(300, 300, "FFFFFF") {
                Sleep 1000
                if A_Index > 30 {
                    break
                }
            }
            Sleep 12000
            continue
        }
        _copy_uid(true)
        if A_Clipboard == "111111111" {
            Click 50, 50
            Sleep 150
            _copy_uid(true)
        }
        _auto_exit()
        if sendtopu(reply)
            last_update_local := A_TickCount
        Sleep 13000
        global last_update
        while A_TickCount - last_update < 60000 { ; 1分延迟
            Sleep 1000
        }
        OutputDebug A_TickCount
        last_update := last_update_local
    }
}

_f3() {
    ; 切区域
    WinGetClientPos(&x, &y, &w, &h)
    Send "m"
    Sleep 750
    Click 1580 * w / 1600, 846 * h / 900 ; 地图
    Sleep 200
    Click 1272 * w / 1600, 327 * h / 900 ; 枫丹
    Sleep 450
    ; 调缩放
    if PixelSearch(&Px, &Py, 37, 365, 36, 534, 0xEDE5DA, 0) && Py >= 440 {
        while not CheckColor(39, 440, "EDE5DA") {
            Send "{WheelUp}"
            Sleep 40
        }
    } else {
        while not CheckColor(39, 457, "EDE5DA") {
            Send "{WheelDown}"
            Sleep 40
        }
    }
    ; 拖地图
    Click 1450 * w / 1600, 373 * h / 900, "Down"
    loop {
        mouseXY(-40, 0)
        Sleep 11
        if A_Index > 10 {
            ; 找锚点
            if PixelSearch(&Px, &Py, 540, 392, 950, 394, 0xFEFEFE, 4) {
                break
            }
            if A_Index > 15
                break
        }
    }
    Sleep 50
    Click "Up"
    if PixelSearch(&Px, &Py, 520, 392, 970, 394, 0xFEFEFE, 4) {
        Sleep 50
        Click "Up"
        Sleep 50
        Click Px, Py, 0 ; 水泽
        ; Sleep 50
        ; Click 1310, 839
    } else {
        Click 788, 395, 0 ; 水泽
    }
}

_copy_uid(flag := false) {
    Send "{F2}"
    Sleep 500
    WinGetClientPos(&x, &y, &w, &h)
    Click 279 * w / 1600, 179 * h / 900
    ; Click 206 * w / 1600, 194 * h / 900 ;测试
    Sleep 200
    Click 488 * w / 1600, 172 * h / 900
    Sleep 300

    cap_and_ocr(x + 516, y + 161, 110 * w / 1600, 23 * w / 1600)
    Sleep 150
    ; Sleep 150
    ; MsgBox(A_Clipboard)

    Send "{Escape}"
    Sleep 80
    if not flag {
        Send "{Escape}"
        Sleep 50
    }
}
_login() {
    SendEvent "+{End}^c"
    WinActivate "原神"
    array := StrSplit(A_Clipboard, "----")
    Click 977, 348
    A_Clipboard := array[1]
    SendEvent "^v"
    Click 991, 420
    A_Clipboard := array[2]
    SendEvent "^v"
    if not CheckColor(582, 512, "DEBC60") {
        Sleep 20
        Click 578, 509
    }
    Sleep 40
    Click 797, 578
}

cap_and_ocr(x, y, w, h) {
    cmd := Format('curl "http://localhost:8888/uid?x={1:d}&y={2:d}&w={3:d}&h={4:d}"', x, y, w, h)
    shell := ComObject("WScript.Shell")
    exec := shell.Run(cmd, 0)
    return
}
cap_and_ocr_text(x, y, w, h) {
    cmd := Format('curl "http://localhost:8888/text?x={1:d}&y={2:d}&w={3:d}&h={4:d}"', x, y, w, h)
    shell := ComObject("WScript.Shell")
    exec := shell.Run(cmd, 0)
    return
}

cap_and_ocr_qq(x, y, w, h) {
    Click 513 * w / 1600, 161 * h / 900, 0
    Sleep 400
    Send "^!a"
    Sleep 150
    MouseClickDrag("Left", 0, 0, w, h, 0, "R")
    Click -137, 23, 1, "Rel"
    Sleep 800
    WinActivate "图片查看器"
    Click 1089, 69
    Sleep 5
    Send "^a^c"
    Click 1240, 12
    Sleep 15
    return A_Clipboard
}

_f2() {
    SendEvent "^c"
    WinActivate "原神"
    WinGetClientPos(&x, &y, &w, &h, "原神")
    if not CheckColor(380 * w / 1600, 844 * h / 900, "ECE5D8") {
        Send "{F2}"
        Sleep 800
    }
    if not CheckColor(1218 * w / 1600, 90 * h / 900, "FFFFFF") {
        Click 1242 * w / 1600, 103 * h / 900
        Sleep 200
    }
    Click 1242 * w / 1600, 103 * h / 900
    Sleep 60
    Click 1403 * w / 1600, 101 * h / 900
    Sleep 100

    if RegExMatch(GetColor(257 * w / 1600, 295 * h / 900), "0xD.D.C.") == 0 {
        Click 1355 * w / 1600, 199 * h / 900
    }
}

sendtopu(reply := "") {
    WinGetClientPos(&x, &y, &w, &h, "QQ频道")
    CoordMode "Pixel", "Screen"
    isSub := CheckColor(x + w - 115, y + h - 36, "1B1B1B")
    CoordMode "Pixel", "Client"
    if A_Clipboard == "111111111"
        return false
    if isSub {
        ; 私车
        if reply != "" {
            _Lusi_text := "#{2}: {1} (re:{3})"
            A_Clipboard := Format(_Lusi_text, A_Clipboard, add_counter_local(), SubStr(reply, 1, 6))
        }
        ControlClick "x" . (w - 120) . " y" . (h - 36), "QQ频道", , "RIGHT" ; 右键
        ControlClick "x" . (w - 84) . " y" . (h - 60), "QQ频道" ; 粘贴
        Sleep 450
        ControlClick "x" . (w - 54) . " y" . (h - 40), "QQ频道"
    } else {
        ; 公车
        c := add_counter_local()
        is_strict := false
        if is_strict {
            _Lusi_text := "{1} 路4自动饭第{2}碗 规则:" . _LineSep . "1. 进去记得打招呼 自动门地主不回复默认不打" . _LineSep . "2. 进去之后贴表情 没贴满3个就是没满 2分钟不满默认炸" . _LineSep . "3. 黑图或者拒绝贴个猴表示炸车" . _LineSep . "4. 优先上前面没满的车" . _LineSep . "5. 只发车不打 发生纠纷与我无关 上车默认同意规则"
            _Lusi_text2 := "{1} 路4自动饭第{2}碗 规则同上"
            if Mod(c, 2) == 1 {
                A_Clipboard := Format(_Lusi_text, A_Clipboard, c)
            } else {
                A_Clipboard := Format(_Lusi_text2, A_Clipboard, c)
            }
        } else {
            _Lusi_text := "{1} 路4第{2}车 发车不打 注意礼貌 (re:{3})"
            A_Clipboard := Format(_Lusi_text, A_Clipboard, c, reply)
        }
        ControlClick "x" . (w - 539) . " y" . (h - 145), "QQ频道", , "RIGHT" ; 右键
        ControlClick "x" . (w - 496) . " y" . (h - 60), "QQ频道" ; 粘贴
        Sleep 450
        ControlClick "x" . (w - 74) . " y" . (h - 36), "QQ频道"
    }
    return true
}
_auto_jinyu() {
    Sleep 333
    SetCapsLockState 1
    BlockInput "On"

    while GetKeyState("CapsLock", "T") {
        ; 进本
        Send "f"
        Sleep 1500
        Click 1426, 846 ; 单人挑战
        Sleep 300
        Click 986, 629 ; 确认
        Sleep 1400
        Click 1426, 846 ; 开始挑战
        ; 检测白屏
        Sleep 2000
        while not CheckColor(837, 536, "3B4255") {
            Sleep 1000
            if A_Index > 100
                break
        }
        Sleep 800
        Click ; 开始
        Sleep 3000
        ysAction("4", true, 500)
        ysAction("2", true, 500)
        ysAction("3", false, 900)
        Send "1"
        Sleep 500
        HoldLeft(1800)
        Sleep 800
        mouseXY(120, -400)
        HoldLeft(2000)
        Sleep 400
        Send "e" ; 6命
        Sleep 500
        mouseXY(0, -500)
        HoldLeft(400)
        Sleep 400
        Send "e" ; 6命
        Sleep 500
        mouseXY(0, -450)
        HoldLeft(400)
        Sleep 1000

        ; 离场
        Send "{Escape}"
        Sleep 450
        Click 1057, 655
        Sleep 2800
        Click 852, 815 ; 点退出
        Sleep 2000
        while CheckColor(477, 266, "FFFFFF") {
            Sleep 1000
            if A_Index > 100
                break
        }
        Sleep 1200
    }
    BlockInput "Off"
    HoldLeft(t) {
        Click "Down"
        Sleep t
        Click "Up"
    }

    ysAction(chr, hold := false, slp := 0) {
        Send chr
        Sleep 300
        if hold {
            Send "{e Down}"
            Sleep 1000
            Send "{e Up}"
        } else {
            Send "e"
        }
        Sleep slp
    }
}
_auto_f2() {
    WinGetClientPos(&x, &y, &w, &h)
    while not (CheckColor(300, 300, "1C1C22") or CheckColor(300, 300, "FFFFFF")) {
        Send "{F2}"
        Sleep 380
        MouseMove 55, 10
        Sleep 420
        loop 6 {
            ; ToolTip(CheckColor(1352, 182 + 104 * (A_Index - 1), "323232"))
            if CheckColor(1352, 158 + 104 * (A_Index - 1), "3.3.3.")
                Click 1349, 197 + 104 * (A_Index - 1)
        }
        Sleep 33
        Send "{Escape}"
        Sleep 350
        if A_Index > 30
            break
    }
}