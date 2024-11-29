#Requires AutoHotkey v2.0
;; 自动执行段
;; 脚本设置
#SingleInstance force
SendMode "Input" ; 设置模拟方式
SetKeyDelay 30, 25 ; SendPlay模式延迟
SetWorkingDir A_ScriptDir ; 设置工作目录
SetTitleMatchMode 3
SetCapsLockState 0
SetWinDelay 20
CoordMode "Mouse"

#Include basic.ahk
#Include yuanshen_c.ahk
#Include gui.ahk
#Include yuanshen_h.ahk


; #HotIf WinActive("原神")
; ~F2:: {
;   A_Clipboard := httpRequest("http://47.94.95.163:8887/r/123/poll?user=🦈")
;   _auto_enter_uniq()
; }


OnClipboardChange WatchClipboard
WatchClipboard(data) {
  if data != 1 {
    return
  }
  text := A_Clipboard

  ; if WinActive("QQ频道") or WinActive("自助餐") or WinActive("WeYS") or WinActive("ahk_exe Cursor.exe") or WinActive("ahk_exe Code.exe") {
  ;   _auto_enter_uniq()
  ; }
  ; if WinActive("WeYS") and RegExMatch(text, "----") {
  ;   _login()
  ; }
}

#HotIf

; F12:: ExitApp


; Alt+中键移动窗口
!MButton::
{
  MouseGetPos &oriX, &oriY, &hwnd
  WinGetPos &winX, &winY, &winW, &winH, hwnd
  Loop
  {
    if !GetKeyState("MButton", "P")
      break
    MouseGetPos &x, &y
    offsetX := x - oriX
    offsetY := y - oriY
    toX := (winX + offsetX)
    toY := (winY + offsetY)
    WinMove toX, toY, , , hwnd
    ToolTip Format("P({1}, {2})", toX, toY)
  }
  ToolTip
}
; Alt+右键缩放窗口
!RButton::
{
  MouseGetPos &oriX, &oriY, &hwnd
  WinGetPos &winX, &winY, &winW, &winH, hwnd
  ; 拖动的坐标如果小于三分之一则从从对应角落开始缩放
  xM := oriX < (winX + winW * 0.33)
  yM := oriY < (winY + winH * 0.33)
  Loop
  {
    if !GetKeyState("RButton", "P")
      break
    MouseGetPos &x, &y
    offsetX := x - oriX
    offsetY := y - oriY
    ; toX := (winW + offsetX)
    ; toY := (winH + offsetY)
    if xM {
      toX := (winW - offsetX)
      pX := (winX + offsetX)
    } else {
      toX := (winW + offsetX)
      pX := winX
    }
    if yM {
      toY := (winH - offsetY)
      pY := (winY + offsetY)
    } else {
      toY := (winH + offsetY)
      pY := winY
    }


    WinMove pX, pY, toX, toY, hwnd
    ToolTip Format("P({1}, {2}) S({3}, {4})", winX, winY, toX, toY)
  }
  ToolTip
}

#HotIf
; 静音当前程序
*Launch_Media:: {
  MouseGetPos(, , &hwnd)
  ; hwnd := GetForegroundWindow()
  p := WinGetProcessName(hwnd)
  msg("静音当前程序：" . p . " (" . GetKeyState("Shift", "P") . ")")
  setProgramVol(p, GetKeyState("Shift", "P") ? 1 : 0)
}

#HotIf WinActive("ahk_exe ZenlessZoneZero.exe")
; r:: {
;   f() {
;     Click
;     Send "eq{Shift}{Space}"
;   }
;   triggerLoop("绝区零战斗", f, 33)
; }

#HotIf WinActive("ahk_exe Client-Win64-Shipping.exe")
y:: {
  f() {
    leftClick1080 1078, 712
    Sleep 300
    leftClick1080 1136, 983
  }

  triggerLoop("合声骸", f, 600)
}

#HotIf WinExist("ahk_exe weys-app.exe") and WinActive("ahk_exe Yuanshen.exe")
F2:: {
  fnSearch() {
    if CheckColor(1222, 104, "FF5C5C") {
      leftClick(1222, 104)
      WaitColor(1224, 101, "FFFFFF")
    }
    leftClick(1222, 104) ; 粘贴
    Sleep 50
    leftClick(1460, 111) ; 搜索
    ; 滚动条
    if !WaitNotHSL(1516, 150, 0xBEB7AF) {
      return
    }
    leftClick(1414, 200) ; 加入
  }
  ; F2
  if CheckColor(397, 843, "ECE5D8") {
    fnSearch()
  }
  ; 单人
  else if CheckColor(303, 24, "9.9.9.") {
    ; msg "单人F2"
    Send "{F2}"
    if !WaitColor(407, 845, "ECE5D8", 2000) {
      msg "失败"
      return
    }
    ; Send "^+R" ; 点加入
    fnSearch()
  }
  ; 2P
  else if CheckColor(304, 51, "96D720") {
    Send "{F2}"
    if !WaitColor(1454, 838, "ECE5D8") {
      return
    }
    leftClick(1454, 838)
    if WaitColor(1061, 630, "4A5366") {
      leftClick(1061, 630)
    }
  } else {
    msg "未知"
  }
}

#HotIf WinActive("ahk_exe Warframe.x64.exe")

XButton2:: {
  Send "^e"
}


CapsLock & t:: {
  f() {
    Send "2"
  }
  triggerLoop("自动按2", f, 33)
}

; WinWatchActivation(titles := [], OnActive?, OnDeactive?) {
;   lastTitle := ""
;   f() {
;     hwnd := GetForegroundWindow()
;     title := WinGetTitle(hwnd)
;     for i, t in titles {
;       if lastTitle != title and t == title {
;         if IsSet(OnActive)
;           OnActive(title)
;       }
;     }
;     lastTitle := title
;   }
;   triggerLoop("WinWatchActivation", f, 33)
; }

; WinWatchActivation(["原神", "WeYS"], (title) => msg(title))