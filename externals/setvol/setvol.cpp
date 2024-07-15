// #include <initguid.h>
#include <Mmdeviceapi.h>
// #include <Functiondiscoverykeys.h>
// #include <Audioclient.h>
#include <Audiopolicy.h>
// #include <endpointvolume.h>
// #include <iostream>
#include <string>

#pragma comment(lib, "ole32.lib")
// #pragma comment(lib, "user32.lib")

std::wstring ExtractFileNameFromIdentifier(WCHAR *identifier)
{
    std::wstring str(identifier);
    // 找到最后一个反斜杠的位置
    size_t lastBackslashPos = str.rfind(L'\\');
    if (lastBackslashPos == std::wstring::npos)
    {
        return L""; // 如果没有找到，返回空字符串
    }

    // 从最后一个反斜杠之后到字符串末尾是文件名
    std::wstring fileName = str.substr(lastBackslashPos + 1);

    // 找到参数的开始位置，即第一个百分号
    size_t paramPos = fileName.find(L'%');
    if (paramPos != std::wstring::npos)
    {
        // 去除参数
        fileName = fileName.substr(0, paramPos);
    }

    return fileName;
}

int wmain(int argc, wchar_t *argv[])
{
    std::wstring programNameToFind;
    float volumeLevel = 0.0f;
    if (argc != 3)
    {
        wprintf(L"Usage: setvol \"program name.exe\" 0.0~1.0\nProgram list:\n", argv[0]);
        // std::wcerr << L"Usage: " << argv[0] << L" <ProgramName> <VolumeLevel>" << std::endl;
        // return -1;
    }
    else
    {
        programNameToFind = std::wstring(argv[1]);
        volumeLevel = std::stof(std::wstring(argv[2]));
    }

    CoInitialize(NULL);

    IMMDeviceEnumerator *deviceEnumerator;
    HRESULT hr = CoCreateInstance(
        __uuidof(MMDeviceEnumerator), NULL,
        CLSCTX_ALL, __uuidof(IMMDeviceEnumerator),
        reinterpret_cast<void **>(&deviceEnumerator));
    if (FAILED(hr))
    {
        // std::cerr << "Failed to create device enumerator" << std::endl;
        CoUninitialize();
        return -1;
    }

    IMMDevice *defaultDevice;
    hr = deviceEnumerator->GetDefaultAudioEndpoint(eRender, eConsole, &defaultDevice);
    deviceEnumerator->Release();
    if (FAILED(hr))
    {
        // std::cerr << "Failed to get default audio endpoint" << std::endl;
        CoUninitialize();
        return -1;
    }

    IAudioSessionManager2 *sessionManager;
    hr = defaultDevice->Activate(__uuidof(IAudioSessionManager2), CLSCTX_ALL,
                                 NULL, reinterpret_cast<void **>(&sessionManager));
    defaultDevice->Release();
    if (FAILED(hr))
    {
        // std::cerr << "Failed to activate session manager" << std::endl;
        CoUninitialize();
        return -1;
    }

    IAudioSessionEnumerator *sessionEnumerator;
    hr = sessionManager->GetSessionEnumerator(&sessionEnumerator);
    sessionManager->Release();
    if (FAILED(hr))
    {
        // std::cerr << "Failed to get session enumerator" << std::endl;
        CoUninitialize();
        return -1;
    }

    int sessionCount;
    hr = sessionEnumerator->GetCount(&sessionCount);
    if (FAILED(hr))
    {
        // std::cerr << "Failed to get session count" << std::endl;
        sessionEnumerator->Release();
        CoUninitialize();
        return -1;
    }

    for (int i = 0; i < sessionCount; ++i)
    {
        IAudioSessionControl *sessionControl;
        hr = sessionEnumerator->GetSession(i, &sessionControl);
        if (SUCCEEDED(hr))
        {
            IAudioSessionControl2 *sessionControl2;
            hr = sessionControl->QueryInterface(__uuidof(IAudioSessionControl2),
                                                (void **)&sessionControl2);
            if (SUCCEEDED(hr))
            {
                WCHAR *sessionInstanceIdentifier;
                hr = sessionControl2->GetSessionInstanceIdentifier(&sessionInstanceIdentifier);
                if (SUCCEEDED(hr) && sessionInstanceIdentifier)
                {
                    auto programName = ExtractFileNameFromIdentifier(sessionInstanceIdentifier);
                    // std::wcout << L"Session " << i << L" (" << programName << L")" << std::endl;
                    if (argc > 2 && programName == programNameToFind)
                    {
                        // Get the simple audio volume interface
                        ISimpleAudioVolume *simpleAudioVolume;
                        hr = sessionControl2->QueryInterface(__uuidof(ISimpleAudioVolume),
                                                             (void **)&simpleAudioVolume);
                        if (SUCCEEDED(hr))
                        {
                            hr = simpleAudioVolume->SetMasterVolume(volumeLevel, NULL);
                            simpleAudioVolume->Release();
                            if (FAILED(hr))
                            {
                                // std::wcerr << L"Failed to set master volume" << std::endl;
                            }
                            else
                            {
                                wprintf(L"Set volume to %f for session %d (%s)\n", volumeLevel, i, programName.c_str());
                                // std::wcout << L"Set volume to " << volumeLevel << L" for session " << i << L" (" << programName << L")" << std::endl;
                            }
                        }
                    }
                    if (argc == 1)
                    {
                        wprintf(L"%d. %s \"%s\" 0\n", i + 1, argv[0], programName.c_str());
                    }
                    CoTaskMemFree(sessionInstanceIdentifier);
                }
                sessionControl2->Release();
            }
            sessionControl->Release();
        }
    }

    sessionEnumerator->Release();
    CoUninitialize();
    return 0;
}
