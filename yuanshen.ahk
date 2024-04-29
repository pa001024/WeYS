#Requires AutoHotkey v2.0
;; 自动执行段
;; 脚本设置
#SingleInstance force
SendMode "Input" ; 设置模拟方式
SetKeyDelay 30, 25 ; SendPlay模式延迟
SetWorkingDir A_ScriptDir ; 设置工作目录
SetTitleMatchMode 3
_LineSep := Chr(13) . Chr(10)

#Include basic.ahk ; 引入常用函数
#Include yuanshen_c.ahk ; 引入常用函数


MyMenu := A_TrayMenu
MyMenu.Add("GUI", on_menu_show_gui)
; MyMenu.AddStandard()
try TraySetIcon("l.ico")

on_menu_show_gui(*) {
  oGui.Show()
  ; oGui.Show("NoActivate")
}

SetCapsLockState 0

Global oGui

AppGui()
; GUI
AppGui() {
  global oGui
  DllCall("shell32\SetCurrentProcessExplicitAppUserModelID", "wstr", "AutoHotkey.WindowSpy")

  ; AlwaysOnTop
  oGui := Gui("Resize MinSize +DPIScale", "YS AHKv2")
  oGui.AddText(, "UID:")
  oGui.AddEdit("w320 r1 ReadOnly vEdit_UID1")
  oGui.AddButton("w320 r1 ReadOnly vBtn_UID1", "Copy").OnEvent("Click", copy_uid)
  oGui.AddEdit("w320 r1 ReadOnly vEdit_UID2")
  oGui.AddButton("w320 r1 ReadOnly vBtn_UID2", "Copy").OnEvent("Click", copy_uid)
  oGui.AddEdit("w320 r1 ReadOnly vEdit_UID3")
  oGui.AddButton("w320 r1 ReadOnly vBtn_UID3", "Copy").OnEvent("Click", copy_uid)
  oGui.AddEdit("w320 r1 ReadOnly vEdit_UID4")
  oGui.AddButton("w320 r1 ReadOnly vBtn_UID4", "Copy").OnEvent("Click", copy_uid)

  ; oGui.Show()
  copy_uid(ctl, *) {
    MsgBox SubStr(ctl.Name, 8)
  }
}

; key binding
CapsLock & r:: Reload()

; 原神
#HotIf WinActive("ahk_exe YuanShen.exe")

CapsLock & F1:: _auto_jinyu() ; 自动刷boss (甘雨竞速版)
CapsLock & `:: _auto_f2() ; 快速F2
; QM
CapsLock & q:: {
  Send "q"
  Sleep 20
  Send "m"
}

global _local_counter := 0
if FileExist("counter.txt")
  _local_counter := Number(FileRead("counter.txt"))

add_counter_local() {
  global _local_counter := _local_counter + 1
  try FileDelete("counter.txt")
  FileAppend(String(_local_counter), "counter.txt")
  return _local_counter
}
reset_counter_local() {
  global _local_counter := 0
  try FileDelete("counter.txt")
  return _local_counter
}

global last_update := 0
; 全自动做饭
CapsLock & F2:: _auto_cook()

; 进地发
CapsLock & 1:: {
  ; _auto_msg("能让我打个精英怪不 o.0") ; 单刷
  _auto_msg("可以让我的几个朋友进来打下枫丹传奇不 ~~ ") ; 一般

  _copy_uid()
  sendtopu("")
  ; _f3()
}
; 私车退出
CapsLock & 2:: {
  _auto_msg("<散olor=#E99697>拜拜~祝你游戏愉快</散olor>")
  _auto_exit()
  _add_counter()
}
_auto_exit() {
  Send "{F2}"
  Sleep 600
  ; if GetColor(1305, 845) == "0xECE5D8"
  Click 1305, 845
}
_auto_msg(txt, auto_exit := true) {
  WinGetClientPos(&x, &y, &w, &h)
  if GetColor(854 * w / 1600, 842 * h / 900) != "0xECE5D8" {
    Send "{Enter}"
    Sleep 120
  }
  if GetColor(642 * w / 1600, 842 * h / 900) != "0xFFFFFF" {
    Send "{Enter}"
    Sleep 250
  }
  SendText2(txt)
  ; SendText2("<散olor=#E99697>拜拜~祝你游戏愉快</散olor>")
  Send "{Enter}"
  Sleep 60
  if auto_exit {
    Send "{Escape}"
    Sleep 140
  }
}
_add_counter() {
  if not WinExist("计数器")
    return
  ControlClick "Static1", "计数器"
}
CapsLock & F4:: {
  PixelSearch(&Px, &Py, 37, 365, 36, 534, 0xEDE5DA, 0)
  MsgBox Py
}
; 传送
F3:: _f3()
; 打完拳皇传送2
F5:: {
  Send "m"
  Sleep 750
  Click 1029, 893, "Down"
  loop {
    mouseXY(-16, -22)
    Sleep 2
    if A_Index > 22
      break
  }
  Click "Up"
  Click 580, 544, 0
}
; 发地
F4:: _copy_uid()


; 一键登录
#HotIf WinActive("ahk_exe Cursor.exe")

XButton2:: _login()
; 一键抢地
#HotIf WinActive("QQ")
XButton2:: {
  Click 2
  _f2()
}
#HotIf WinActive("QQ频道")
XButton2:: {
  Click 2
  SendEvent "^c"
  _f2()
}
F2:: _f2()
#HotIf