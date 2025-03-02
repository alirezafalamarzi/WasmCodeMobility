#include "myworld.h"
#include <stdlib.h>
void exports_myworld_to_string(myworld_user_data_t *user1, myworld_user_data_t *ret) {
    ret = malloc(sizeof(myworld_user_data_t));
    myworld_string_dup(&(ret->first_name), "Mike");
    myworld_string_dup(&(ret->first_name), "Jordan");
    ret->age = 32;
    unsigned int *grades = malloc(sizeof(unsigned int) * 3);
    ret->grades.ptr = grades;
    ret->grades.len = 3;
}