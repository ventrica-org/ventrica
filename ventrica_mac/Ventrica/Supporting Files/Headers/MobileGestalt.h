#ifndef LIBMOBILEGESTALT_H_
#define LIBMOBILEGESTALT_H_

#include <CoreFoundation/CoreFoundation.h>

#if __cplusplus
extern "C" {
#endif

CFPropertyListRef MGCopyAnswer(CFStringRef property);
static const CFStringRef kMGPhysicalHardwareNameString = CFSTR("PhysicalHardwareNameString");

#if __cplusplus
}
#endif

#endif /* LIBMOBILEGESTALT_H_ */
